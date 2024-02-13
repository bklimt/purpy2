#![allow(clippy::collapsible_else_if)]

use std::cmp::Ordering;
use std::mem;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use anyhow::{Context, Result};
use log::{debug, info, log_enabled};
use num_traits::Zero;

use crate::constants::{
    COYOTE_TIME, FALL_ACCELERATION, JUMP_ACCELERATION, JUMP_GRACE_TIME, JUMP_INITIAL_SPEED,
    PLAYER_DEFAULT_X, PLAYER_DEFAULT_Y, SLIDE_SPEED_DECELERATION, SPRING_BOUNCE_DURATION,
    SPRING_BOUNCE_VELOCITY, SPRING_JUMP_DURATION, SPRING_JUMP_VELOCITY, TARGET_WALK_SPEED,
    TOAST_HEIGHT, TOAST_SPEED, TOAST_TIME, VIEWPORT_PAN_SPEED, WALK_SPEED_ACCELERATION,
    WALK_SPEED_DECELERATION, WALL_JUMP_HORIZONTAL_SPEED, WALL_JUMP_VERTICAL_SPEED,
    WALL_SLIDE_SPEED, WALL_SLIDE_TIME, WALL_STICK_TIME,
};
use crate::door::Door;
use crate::filemanager::FileManager;
use crate::font::Font;
use crate::geometry::{Pixels, Point, Rect, Subpixels};
use crate::imagemanager::ImageLoader;
use crate::inputmanager::InputSnapshot;
use crate::menu::Menu;
use crate::platform::{Bagel, Button, Conveyor, MovingPlatform, Platform, PlatformType, Spring};
use crate::player::{Player, PlayerState};
use crate::rendercontext::{RenderContext, RenderLayer};
use crate::scene::{Scene, SceneResult};
use crate::smallintset::SmallIntSet;
use crate::soundmanager::{Sound, SoundManager};
use crate::star::Star;
use crate::switchstate::SwitchState;
use crate::tilemap::{TileIndex, TileMap};
use crate::tileset::TileProperties;
use crate::utils::{cmp_in_direction, Color, Direction};
use crate::warp::Warp;

struct PlatformIntersectionResult {
    offset: Subpixels,
    platforms: SmallIntSet<usize>,
}

// The results of trying to move.
struct TryMovePlayerResult {
    offset: Subpixels,
    tile_ids: SmallIntSet<TileIndex>,
    platforms: SmallIntSet<usize>,
}

struct MoveAndCheckResult {
    on_ground: bool,
    on_tile_ids: SmallIntSet<TileIndex>,
    on_platforms: SmallIntSet<usize>,
    hit_ceiling: bool,
    against_wall: bool,
    crushed_by_platform: bool,
    stuck_in_wall: bool,
}

struct MovePlayerXResult {
    pushing_against_wall: bool,
    stuck_in_wall: bool,
    crushed_by_platform: bool,
}

struct MovePlayerYResult {
    on_ground: bool,
    _platforms: SmallIntSet<usize>,
    _tile_ids: SmallIntSet<TileIndex>,
    stuck_in_wall: bool,
    crushed_by_platform: bool,
}

#[derive(Debug, Clone, Copy)]
struct PlayerMovementResult {
    on_ground: bool,
    pushing_against_wall: bool,
    jump_down: bool,
    jump_triggered: bool,
    crouch_down: bool,
    _stuck_in_wall: bool,
    crushed_by_platform: bool,
}

pub struct Level {
    _name: String,
    map_path: PathBuf,
    map: Rc<TileMap>,
    player: Player,

    pause_menu: Menu,
    paused: bool,

    wall_stick_counter: i32,
    wall_stick_facing_right: bool,
    wall_slide_counter: i32,

    coyote_counter: i32,
    jump_grace_counter: i32,
    spring_counter: i32,

    previous_map_offset: Option<Point<Subpixels>>,
    toast_text: String,
    toast_position: Subpixels,
    toast_counter: i32,

    // platforms, stars, and doors
    platforms: Vec<Platform>,
    stars: Vec<Star>,
    doors: Vec<Door>,
    warps: Vec<Warp>,

    star_count: i32,
    current_platform: Option<usize>,
    current_slopes: SmallIntSet<TileIndex>,
    switches: SwitchState,
    current_switch_tiles: SmallIntSet<TileIndex>,
    current_door: Option<usize>,

    previous_transition: String,
}

fn inc_player_x(player: &mut Player, offset: Subpixels) {
    player.position.x += offset;
}

fn inc_player_y(player: &mut Player, offset: Subpixels) {
    player.position.y += offset;
}

impl Level {
    pub fn new(
        map_path: &Path,
        files: &FileManager,
        images: &mut dyn ImageLoader,
    ) -> Result<Level> {
        let pause_menu = Menu::new_menu(Path::new("assets/menus/pause.tmx"), files, images)?;
        let paused = false;

        let wall_stick_counter = WALL_STICK_TIME;
        let wall_stick_facing_right = false;
        let wall_slide_counter = WALL_SLIDE_TIME;

        let coyote_counter = COYOTE_TIME;
        let jump_grace_counter = 0;
        let spring_counter = 0;

        let toast_position = TOAST_HEIGHT * -1;
        let toast_counter = TOAST_TIME;

        let name: String = map_path
            .file_stem()
            .and_then(|s| s.to_str())
            .context("invalid filename")?
            .to_string();
        let toast_text = name.clone();
        let previous_map_offset = None;
        let map = Rc::new(TileMap::from_file(map_path, files, images)?);
        let mut player = Player::new(files, images)?;
        player.position.x = PLAYER_DEFAULT_X;
        player.position.y = PLAYER_DEFAULT_Y;

        let star_count = 0;
        let switches = SwitchState::new();
        let current_switch_tiles = SmallIntSet::new();
        let current_slopes = SmallIntSet::new();
        let current_platform = None;
        let current_door = None;

        let mut platforms: Vec<Platform> = Vec::new();
        let mut stars = Vec::new();
        let mut doors = Vec::new();
        let mut warps = Vec::new();

        for obj in map.objects.iter() {
            if obj.properties.platform {
                platforms.push(MovingPlatform::new(obj, map.clone())?);
            }
            if obj.properties.bagel {
                platforms.push(Bagel::new(obj, map.clone())?);
            }
            if obj.properties.convey.is_some() {
                platforms.push(Conveyor::new(obj, map.clone())?);
            }
            if obj.properties.spring {
                platforms.push(Spring::new(obj, map.clone(), images)?);
            }
            if obj.properties.button {
                platforms.push(Button::new(obj, map.clone(), images)?);
            }
            if obj.properties.spawn {
                player.position.x = obj.position.x.as_subpixels();
                player.position.y = obj.position.y.as_subpixels();
                player.delta.x = obj.properties.dx.as_subpixels();
                player.delta.y = obj.properties.dy.as_subpixels();
                player.state = PlayerState::Jumping;
                player.facing_right = !obj.properties.facing_left;
            }
            if obj.properties.door {
                doors.push(Door::new(obj, images)?);
            }
            if obj.properties.star {
                stars.push(Star::new(obj, map.clone())?);
            }
            if obj.properties.warp.is_some() {
                warps.push(Warp::new(obj)?);
            }
        }

        let map_path = map_path.to_owned();
        let previous_transition = "".to_owned();

        Ok(Level {
            _name: name,
            map_path,
            map,
            player,
            pause_menu,
            paused,
            wall_stick_counter,
            wall_stick_facing_right,
            wall_slide_counter,
            coyote_counter,
            jump_grace_counter,
            spring_counter,
            previous_map_offset,
            toast_text,
            toast_position,
            toast_counter,
            platforms,
            stars,
            doors,
            warps,
            star_count,
            current_platform,
            current_slopes,
            switches,
            current_switch_tiles,
            current_door,
            previous_transition,
        })
    }
}

impl Level {
    /*
     * Movement.
     */

    fn update_player_trajectory_x(&mut self, inputs: &InputSnapshot) {
        if matches!(self.player.state, PlayerState::Crouching) {
            match self.player.delta.x.cmp(&Subpixels::zero()) {
                Ordering::Greater => {
                    self.player.delta.x =
                        (self.player.delta.x - SLIDE_SPEED_DECELERATION).max(Subpixels::zero());
                }
                Ordering::Less => {
                    self.player.delta.x =
                        (self.player.delta.x + SLIDE_SPEED_DECELERATION).min(Subpixels::zero());
                }
                Ordering::Equal => {}
            }
            return;
        }

        // Apply controller input.
        let mut target_dx = Subpixels::zero();
        if inputs.player_left_down && !inputs.player_right_down {
            target_dx = TARGET_WALK_SPEED * -1;
        } else if inputs.player_right_down && !inputs.player_left_down {
            target_dx = TARGET_WALK_SPEED;
        }

        // Change the velocity toward the target velocity.
        match self.player.delta.x.cmp(&Subpixels::zero()) {
            Ordering::Greater => {
                // We're facing right.
                if target_dx > self.player.delta.x {
                    self.player.delta.x += WALK_SPEED_ACCELERATION;
                    self.player.delta.x = self.player.delta.x.min(target_dx);
                }
                if target_dx < self.player.delta.x {
                    self.player.delta.x -= WALK_SPEED_DECELERATION;
                    self.player.delta.x = self.player.delta.x.max(target_dx);
                }
            }
            Ordering::Less => {
                // We're facing left.
                if target_dx > self.player.delta.x {
                    self.player.delta.x += WALK_SPEED_DECELERATION;
                    self.player.delta.x = self.player.delta.x.min(target_dx);
                }
                if target_dx < self.player.delta.x {
                    self.player.delta.x -= WALK_SPEED_ACCELERATION;
                    self.player.delta.x = self.player.delta.x.max(target_dx);
                }
            }
            Ordering::Equal => {
                // We're stopped.
                if target_dx > self.player.delta.x {
                    self.player.delta.x += WALK_SPEED_ACCELERATION;
                    self.player.delta.x = self.player.delta.x.min(target_dx);
                }
                if target_dx < self.player.delta.x {
                    self.player.delta.x -= WALK_SPEED_ACCELERATION;
                    self.player.delta.x = self.player.delta.x.max(target_dx);
                }
            }
        }
    }

    fn update_player_trajectory_y(&mut self) {
        let gravity = self.map.get_gravity();
        match self.player.state {
            PlayerState::Standing | PlayerState::Crouching => {
                // Fall at least one pixel so that we hit the ground again.
                self.player.delta.y = self.player.delta.y.max(Subpixels::new(1));
            }
            PlayerState::Jumping => {
                // Apply gravity.
                if self.player.delta.y < gravity {
                    self.player.delta.y += JUMP_ACCELERATION;
                }
                self.player.delta.y = self.player.delta.y.min(gravity);
            }
            PlayerState::Falling => {
                // Apply gravity.
                if self.player.delta.y < gravity {
                    self.player.delta.y += FALL_ACCELERATION;
                }
                self.player.delta.y = self.player.delta.y.min(gravity);
            }
            PlayerState::WallSliding => {
                // When you first grab the wall, don't start sliding for a while.
                if self.wall_slide_counter > 0 {
                    self.wall_slide_counter -= 1;
                    self.player.delta.y = Subpixels::zero();
                } else {
                    self.player.delta.y = WALL_SLIDE_SPEED;
                }
            }
            PlayerState::Stopped => {}
        }
    }

    fn find_platform_intersections(
        &self,
        player_rect: Rect<Subpixels>,
        direction: Direction,
        is_backwards: bool,
    ) -> PlatformIntersectionResult {
        let mut result = PlatformIntersectionResult {
            offset: Subpixels::zero(),
            platforms: SmallIntSet::new(),
        };
        for (i, platform) in self.platforms.iter().enumerate() {
            let distance = platform.try_move_to(player_rect, direction, is_backwards);
            if distance.is_zero() {
                continue;
            }

            match cmp_in_direction(distance, result.offset, direction) {
                Ordering::Less => {
                    result.offset = distance;
                    result.platforms = SmallIntSet::new();
                    result.platforms.insert(i);
                }
                Ordering::Equal => {
                    result.platforms.insert(i);
                }
                Ordering::Greater => {}
            }
        }
        result
    }

    // Returns how far this player needs to move in direction to not intersect, in sub-pixels.
    fn try_move_player(&self, direction: Direction, is_backwards: bool) -> TryMovePlayerResult {
        let player_rect = self.player.get_target_bounds_rect(Some(direction));

        let map_result = self
            .map
            .try_move_to(player_rect, direction, &self.switches, is_backwards);
        let platform_result =
            self.find_platform_intersections(player_rect, direction, is_backwards);

        match cmp_in_direction(platform_result.offset, map_result.hard_offset, direction) {
            Ordering::Less | Ordering::Equal => TryMovePlayerResult {
                offset: platform_result.offset,
                platforms: platform_result.platforms,
                tile_ids: SmallIntSet::new(),
            },
            Ordering::Greater => TryMovePlayerResult {
                offset: map_result.hard_offset,
                platforms: SmallIntSet::new(),
                tile_ids: map_result.tile_ids,
            },
        }
    }

    // Returns whether the first move hit a wall or platform.
    fn move_and_check(
        &mut self,
        forward: Direction,
        apply_offset: fn(&mut Player, Subpixels) -> (),
    ) -> MoveAndCheckResult {
        let move_result1 = self.try_move_player(forward, false);
        apply_offset(&mut self.player, move_result1.offset);

        // Try the opposite direction.
        let move_result2 = self.try_move_player(forward.opposite(), true);
        let offset = move_result2.offset;
        apply_offset(&mut self.player, offset);

        let mut hit_solid_platform1 = false;
        for platform in move_result1.platforms.iter() {
            if self.platforms[*platform].is_solid() {
                hit_solid_platform1 = true;
            }
        }
        let mut hit_solid_platform2 = false;
        for platform in move_result2.platforms.iter() {
            if self.platforms[*platform].is_solid() {
                hit_solid_platform2 = true;
            }
        }

        let mut result = MoveAndCheckResult {
            on_ground: false,
            on_tile_ids: SmallIntSet::new(),
            on_platforms: SmallIntSet::new(),
            hit_ceiling: false,
            against_wall: false,
            crushed_by_platform: false,
            stuck_in_wall: false,
        };
        match forward {
            Direction::Down => {
                result.on_ground = !move_result1.offset.is_zero();
                result.on_tile_ids = move_result1.tile_ids;
                result.on_platforms = move_result1.platforms;
            }
            Direction::Up => {
                // If we're traveling up, then if we hit something below, it's not the ground,
                // unless we're standing on a platform.
                if !matches!(self.player.state, PlayerState::Jumping)
                    && !matches!(self.player.state, PlayerState::Falling)
                {
                    result.on_ground = !move_result2.offset.is_zero();
                }
                result.hit_ceiling = !move_result1.offset.is_zero();
                result.on_tile_ids = move_result2.tile_ids;
                result.on_platforms = move_result2.platforms;
            }
            Direction::Left | Direction::Right => {
                result.against_wall = !move_result1.offset.is_zero()
            }
        }

        // See if we're crushed.
        if !offset.is_zero() {
            let crush_check = self.try_move_player(forward, false);
            if !crush_check.offset.is_zero() {
                let crushed = hit_solid_platform1 || hit_solid_platform2;
                if crushed {
                    result.crushed_by_platform = true;
                } else {
                    result.stuck_in_wall = true;
                }
            }
        }

        result
    }

    fn move_player_x(&mut self, inputs: &InputSnapshot) -> MovePlayerXResult {
        let mut dx = self.player.delta.x;
        if let Some(current_platform) = self.current_platform {
            dx += self.platforms[current_platform].dx();
        }
        self.player.position.x += dx;

        let (move_result, pushing) =
            if dx < Subpixels::zero() || (dx.is_zero() && !self.player.facing_right) {
                // Moving left.
                let move_result = self.move_and_check(Direction::Left, inc_player_x);
                let pushing = inputs.player_left_down;
                (move_result, pushing)
            } else {
                // Moving right.
                let move_result = self.move_and_check(Direction::Right, inc_player_x);
                let pushing = inputs.player_right_down;
                (move_result, pushing)
            };

        let result = MovePlayerXResult {
            pushing_against_wall: pushing && move_result.against_wall,
            crushed_by_platform: move_result.crushed_by_platform,
            stuck_in_wall: move_result.stuck_in_wall,
        };

        // If you're against the wall, you're stopped.
        if result.pushing_against_wall {
            self.player.delta.x = Subpixels::zero();
        }

        result
    }

    fn get_slope_dy(&self) -> Subpixels {
        let mut slope_fall = Subpixels::zero();
        for slope_id in self.current_slopes.iter() {
            let slope = self.map.get_slope(*slope_id).expect("must be valid");
            let left_y = slope.left_y;
            let right_y = slope.right_y;
            let mut fall: Subpixels = Subpixels::zero();
            if self.player.delta.x > Subpixels::zero()
                || (self.player.delta.x.is_zero() && self.player.facing_right)
            {
                // The player is facing right.
                if right_y > left_y {
                    fall = right_y - left_y;
                }
            } else {
                // The player is facing left.
                if left_y > right_y {
                    fall = left_y - right_y;
                }
            }
            slope_fall = slope_fall.max(fall);
        }
        slope_fall
    }

    fn move_player_y(&mut self, sounds: &mut SoundManager) -> MovePlayerYResult {
        let mut dy = self.player.delta.y;
        if let Some(current_platform) = self.current_platform {
            // This could be positive or negative.
            dy += self.platforms[current_platform].dy();
        }

        // If you're on a slope, make sure to fall at least the slope amount.
        if dy >= Subpixels::zero() {
            dy = dy.max(self.get_slope_dy());
        }

        self.player.position.y += dy;

        if dy <= Subpixels::zero() {
            // Moving up.
            let move_result = self.move_and_check(Direction::Up, inc_player_y);
            if move_result.hit_ceiling {
                self.player.delta.y = Subpixels::zero();
            }

            self.handle_slopes(&move_result.on_tile_ids);
            self.handle_current_platforms(&move_result.on_platforms);

            MovePlayerYResult {
                on_ground: move_result.on_ground,
                crushed_by_platform: move_result.crushed_by_platform,
                stuck_in_wall: move_result.stuck_in_wall,
                _platforms: SmallIntSet::new(),
                _tile_ids: SmallIntSet::new(),
            }
        } else {
            // Moving down.
            let move_result = self.move_and_check(Direction::Down, inc_player_y);

            self.handle_spikes(&move_result.on_tile_ids);
            self.handle_switch_tiles(&move_result.on_tile_ids, sounds);
            self.handle_slopes(&move_result.on_tile_ids);
            self.handle_current_platforms(&move_result.on_platforms);

            MovePlayerYResult {
                on_ground: move_result.on_ground,
                _tile_ids: move_result.on_tile_ids,
                _platforms: move_result.on_platforms,
                crushed_by_platform: move_result.crushed_by_platform,
                stuck_in_wall: move_result.stuck_in_wall,
            }
        }
    }

    fn handle_slopes(&mut self, tiles: &SmallIntSet<TileIndex>) {
        self.current_slopes.clear();
        for tile_id in tiles.iter() {
            if let Some(TileProperties { slope: true, .. }) = self.map.get_tile_properties(*tile_id)
            {
                self.current_slopes.insert(*tile_id);
            }
        }
    }

    fn handle_spikes(&mut self, tiles: &SmallIntSet<TileIndex>) {
        for tile_id in tiles.iter() {
            if let Some(TileProperties { deadly: true, .. }) =
                self.map.get_tile_properties(*tile_id)
            {
                self.player.is_dead = true;
            }
        }
    }

    fn handle_current_platforms(&mut self, platforms: &SmallIntSet<usize>) {
        self.current_platform = None;
        for platform in self.platforms.iter_mut() {
            platform.set_occupied(false);
        }

        for platform_index in platforms.iter() {
            let platform = &mut self.platforms[*platform_index];

            platform.set_occupied(true);
            // TODO: Be smarter about what platform we pick.
            self.current_platform = Some(*platform_index);
        }
    }

    fn handle_switch_tiles(&mut self, tiles: &SmallIntSet<TileIndex>, sounds: &mut SoundManager) {
        let new_switch_tiles = SmallIntSet::new();
        let previous = mem::replace(&mut self.current_switch_tiles, new_switch_tiles);
        for t in tiles.iter() {
            let Some(TileProperties {
                switch: Some(switch),
                ..
            }) = self.map.get_tile_properties(*t)
            else {
                continue;
            };
            self.current_switch_tiles.insert(*t);
            if previous.contains(*t) {
                continue;
            }
            sounds.play(Sound::Click);
            self.switches.apply_command(switch);
        }
    }

    fn update_player_movement(
        &mut self,
        inputs: &InputSnapshot,
        sounds: &mut SoundManager,
    ) -> PlayerMovementResult {
        self.update_player_trajectory_x(inputs);
        self.update_player_trajectory_y();

        let x_result = self.move_player_x(inputs);
        let y_result = self.move_player_y(sounds);

        PlayerMovementResult {
            on_ground: y_result.on_ground,
            pushing_against_wall: x_result.pushing_against_wall,
            jump_down: inputs.player_jump_down,
            jump_triggered: inputs.player_jump_clicked,
            crouch_down: inputs.player_crouch_down,
            _stuck_in_wall: x_result.stuck_in_wall || y_result.stuck_in_wall,
            crushed_by_platform: x_result.crushed_by_platform || y_result.crushed_by_platform,
        }
    }

    fn update_player_state(&mut self, movement: PlayerMovementResult) {
        if movement.on_ground {
            self.coyote_counter = COYOTE_TIME;
        } else if self.coyote_counter > 0 {
            self.coyote_counter -= 1;
        }

        if self.jump_grace_counter > 0 {
            self.jump_grace_counter -= 1;
        }

        if movement.crushed_by_platform {
            self.player.state = PlayerState::Stopped;
            self.player.is_dead = true;
        } else {
            match self.player.state {
                PlayerState::Stopped => {}
                PlayerState::Standing => {
                    let launch = if let Some(Platform {
                        subtype: PlatformType::Spring(spring),
                        ..
                    }) = self.current_platform.map(|i| &self.platforms[i])
                    {
                        spring.launch
                    } else {
                        false
                    };
                    if launch {
                        self.jump_grace_counter = 0;
                        self.player.state = PlayerState::Jumping;
                        if movement.jump_triggered || self.jump_grace_counter > 0 {
                            self.spring_counter = SPRING_JUMP_DURATION;
                            self.player.delta.y = SPRING_JUMP_VELOCITY * -1;
                        } else {
                            self.spring_counter = SPRING_BOUNCE_DURATION;
                            self.player.delta.y = SPRING_BOUNCE_VELOCITY * -1;
                        }
                    } else if self.coyote_counter == 0 {
                        self.player.state = PlayerState::Falling;
                        self.player.delta.y = Subpixels::zero();
                        if let Some(current_platform) = self.current_platform {
                            self.player.delta.x = self.platforms[current_platform].dx();
                        }
                    } else if movement.crouch_down {
                        self.player.state = PlayerState::Crouching;
                    } else if movement.jump_triggered || self.jump_grace_counter > 0 {
                        if let Some(current_door) = self.current_door {
                            if self.doors[current_door].is_open() {
                                self.player.state = PlayerState::Stopped;
                                self.doors[current_door].close();
                            }
                        } else {
                            self.jump_grace_counter = 0;
                            self.player.state = PlayerState::Jumping;
                            let should_boost = if let Some(Platform {
                                subtype: PlatformType::Spring(spring),
                                ..
                            }) =
                                self.current_platform.map(|i| &self.platforms[i])
                            {
                                spring.should_boost()
                            } else {
                                false
                            };
                            if should_boost {
                                self.spring_counter = SPRING_JUMP_DURATION;
                                self.player.delta.y = SPRING_JUMP_VELOCITY * -1;
                            } else {
                                self.spring_counter = 0;
                                self.player.delta.y = JUMP_INITIAL_SPEED * -1;
                            }
                            if let Some(current_platform) = self.current_platform {
                                self.player.delta.x += self.platforms[current_platform].dx();
                            }
                        }
                    }
                }
                PlayerState::Falling => {
                    if movement.jump_triggered {
                        self.jump_grace_counter = JUMP_GRACE_TIME;
                    }
                    if movement.on_ground {
                        self.player.state = PlayerState::Standing;
                        self.player.delta.y = Subpixels::zero();
                    } else {
                        if movement.pushing_against_wall && self.player.delta.y >= Subpixels::zero()
                        {
                            self.player.state = PlayerState::WallSliding;
                            self.wall_slide_counter = WALL_SLIDE_TIME;
                        }
                    }
                }
                PlayerState::Jumping => {
                    if movement.on_ground {
                        self.player.state = PlayerState::Standing;
                        self.player.delta.y = Subpixels::zero();
                    } else if self.player.delta.y >= Subpixels::zero() {
                        self.player.state = PlayerState::Falling;
                    } else {
                        if !movement.jump_down {
                            if self.spring_counter == 0 {
                                self.player.state = PlayerState::Falling;
                                self.player.delta.y = Subpixels::zero();
                            } else {
                                self.spring_counter -= 1;
                            }
                        }
                    }
                }
                PlayerState::WallSliding => {
                    if movement.jump_triggered {
                        self.player.state = PlayerState::Jumping;
                        self.player.delta.y = WALL_JUMP_VERTICAL_SPEED * -1;
                        if self.player.facing_right {
                            self.player.delta.x = WALL_JUMP_HORIZONTAL_SPEED * -1;
                        } else {
                            self.player.delta.x = WALL_JUMP_HORIZONTAL_SPEED;
                        }
                    } else if movement.on_ground {
                        self.player.state = PlayerState::Standing;
                    } else if movement.pushing_against_wall {
                        self.wall_stick_counter = WALL_STICK_TIME;
                        self.wall_stick_facing_right = self.player.facing_right;
                    } else {
                        if self.wall_stick_facing_right != self.player.facing_right {
                            self.player.state = PlayerState::Falling;
                        } else if self.wall_stick_counter > 0 {
                            self.wall_stick_counter -= 1;
                        } else {
                            self.player.state = PlayerState::Falling;
                        }
                    }
                }
                PlayerState::Crouching => {
                    if !movement.on_ground {
                        self.player.state = PlayerState::Falling;
                        self.player.delta.y = Subpixels::zero();
                    } else if !movement.crouch_down {
                        self.player.state = PlayerState::Standing;
                    }
                }
            }
        }
    }
}

impl Scene for Level {
    fn update(&mut self, inputs: &InputSnapshot, sounds: &mut SoundManager) -> SceneResult {
        if self.paused {
            return self.pause_menu.update(inputs, sounds);
        }

        if inputs.cancel_clicked {
            self.paused = true;
            return SceneResult::Continue;
        }

        for platform in self.platforms.iter_mut() {
            platform.update(&mut self.switches, sounds);
        }

        let movement = match self.player.state {
            PlayerState::Stopped => PlayerMovementResult {
                on_ground: false,
                pushing_against_wall: false,
                jump_down: false,
                jump_triggered: false,
                crouch_down: false,
                _stuck_in_wall: false,
                crushed_by_platform: false,
            },
            _ => self.update_player_movement(inputs, sounds),
        };

        let start_state: PlayerState = self.player.state;
        self.update_player_state(movement);
        self.player
            .update_sprite()
            .expect("state machine should be valid");

        // Make sure you aren't stuck in a wall.
        let player_rect = self.player.get_target_bounds_rect(None);

        self.current_door = None;
        for (i, door) in self.doors.iter_mut().enumerate() {
            door.update(player_rect, self.star_count);
            if door.is_closed() {
                return SceneResult::SwitchToLevel {
                    path: door
                        .destination
                        .as_ref()
                        .map(|s| Path::new(s).to_owned())
                        .unwrap_or(self.map_path.clone()),
                };
            }
            if door.active {
                self.current_door = Some(i);
            }
        }

        for warp in self.warps.iter() {
            if warp.is_inside(player_rect) {
                return SceneResult::SwitchToLevel {
                    path: Path::new(&warp.destination).to_owned(),
                };
            }
        }

        let old_stars = mem::take(&mut self.stars);
        for star in old_stars.into_iter() {
            if star.intersects(player_rect) {
                sounds.play(Sound::Star);
                self.star_count += 1;
                self.toast_text = format!("STARS x {}", self.star_count);
                self.toast_counter = TOAST_TIME;
            } else {
                self.stars.push(star);
            }
        }

        if log_enabled!(log::Level::Info) {
            // TODO: Include slopes.
            let attribs = format!(
                "{:?}, idle={}, platform={:?}",
                movement, self.player.is_idle, self.current_platform,
            );
            let transition = format!("{:?} x {} -> {:?}", start_state, attribs, self.player.state);
            if transition != self.previous_transition {
                self.previous_transition = transition;
                info!("{}", self.previous_transition);
            }
        }

        if self.player.is_dead {
            return SceneResult::SwitchToKillScreen {
                path: self.map_path.clone(),
            };
        }

        if self.toast_counter == 0 {
            if self.toast_position > TOAST_HEIGHT * -1 {
                self.toast_position -= TOAST_SPEED;
            }
        } else {
            self.toast_counter -= 1;
            if self.toast_position < Subpixels::zero() {
                self.toast_position += TOAST_SPEED;
            }
        }

        if log_enabled!(log::Level::Debug) {
            debug!("Level state: {:?}", self.player);
        }

        SceneResult::Continue
    }

    fn draw(&mut self, context: &mut RenderContext, font: &Font) {
        let dest = context.logical_area_in_subpixels();

        // Make sure the player is on the screen, and then center them if possible.
        let player_rect = self.player.get_target_bounds_rect(None);
        let (preferred_x, preferred_y) = self.map.get_preferred_view(player_rect);
        let player = self.player.position;
        let mut player_draw = Point::new(dest.w / 2, dest.h / 2);
        // Don't waste space on the sides of the screen beyond the map.
        if player_draw.x > player.x {
            player_draw.x = player.x;
        }
        // The map is drawn 4 pixels from the top of the screen.
        if player_draw.y > player.y + Pixels::new(4).as_subpixels() {
            player_draw.y = player.y + Pixels::new(4).as_subpixels();
        }
        let right_limit = dest.w - self.map.tilewidth.as_subpixels() * self.map.width;
        if player_draw.x < player.x + right_limit {
            player_draw.x = player.x + right_limit;
        }
        let bottom_limit = dest.h - self.map.tileheight.as_subpixels() * self.map.height;
        if player_draw.y < player.y + bottom_limit {
            player_draw.y = player.y + bottom_limit;
        }
        let mut map_offset = player_draw - player;

        if let Some(preferred_x) = preferred_x {
            map_offset = (preferred_x * -1, map_offset.y).into();
            player_draw.x = player.x + map_offset.x;
        }
        if let Some(preferred_y) = preferred_y {
            map_offset = (map_offset.x, preferred_y * -1).into();
            player_draw.y = player.y + map_offset.y;
        }

        // Don't let the viewport move too much in between frames.
        if let Some(prev) = self.previous_map_offset.as_ref() {
            if (map_offset.x - prev.x).abs() > VIEWPORT_PAN_SPEED {
                map_offset.x = match prev.x.cmp(&map_offset.x) {
                    Ordering::Less => prev.x + VIEWPORT_PAN_SPEED,
                    Ordering::Greater => prev.x - VIEWPORT_PAN_SPEED,
                    Ordering::Equal => map_offset.x,
                };
                player_draw.x = player.x + map_offset.x;
            }
            if (map_offset.y - prev.y).abs() > VIEWPORT_PAN_SPEED {
                map_offset.y = match prev.y.cmp(&map_offset.y) {
                    Ordering::Less => prev.y + VIEWPORT_PAN_SPEED,
                    Ordering::Greater => prev.y - VIEWPORT_PAN_SPEED,
                    Ordering::Equal => map_offset.y,
                };
                player_draw.y = player.y + map_offset.y;
            }
        }
        self.previous_map_offset = Some(map_offset);

        // Do the actual drawing.
        self.map.draw_background(
            context,
            RenderLayer::Player,
            dest,
            map_offset,
            &self.switches,
        );
        for door in self.doors.iter() {
            door.draw_background(context, RenderLayer::Player, map_offset, font);
        }
        for platform in self.platforms.iter() {
            platform.draw(context, RenderLayer::Player, map_offset);
        }
        for star in self.stars.iter() {
            star.draw(context, RenderLayer::Player, map_offset);
        }
        self.player.draw(context, RenderLayer::Player, player_draw);
        for door in self.doors.iter() {
            door.draw_foreground(context, RenderLayer::Player, map_offset);
        }
        self.map.draw_foreground(
            context,
            RenderLayer::Player,
            dest,
            map_offset,
            &self.switches,
        );

        // Draw the text overlay.
        let top_bar_bgcolor = Color {
            r: 0,
            g: 0,
            b: 0,
            a: 127,
        };
        let top_bar_area = Rect {
            x: dest.x,
            y: dest.y + self.toast_position,
            w: dest.w,
            h: TOAST_HEIGHT,
        };
        if top_bar_area.bottom() > Subpixels::zero() {
            context.fill_rect(top_bar_area, RenderLayer::Hud, top_bar_bgcolor);
            let text_offset = Point::new(Pixels::new(2), Pixels::new(2));
            let text_offset: Point<Subpixels> = text_offset.into();
            font.draw_string(
                context,
                RenderLayer::Hud,
                top_bar_area.top_left() + text_offset,
                &self.toast_text,
            );
        }

        context.is_dark = self.map.properties.dark;

        let spotlight_pos =
            player_draw + Point::new(Subpixels::from_pixels(12), Subpixels::from_pixels(12));

        let spotlight_radius = Subpixels::from_pixels(120);
        context.add_light(spotlight_pos, spotlight_radius);

        if self.paused {
            self.pause_menu.draw(context, font);
        }
    }
}
