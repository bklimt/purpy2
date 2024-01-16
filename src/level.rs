use std::mem;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use sdl2::render::RenderTarget;

use crate::constants::{
    COYOTE_TIME, FALL_ACCELERATION, FALL_MAX_GRAVITY, JUMP_ACCELERATION, JUMP_GRACE_TIME,
    JUMP_INITIAL_SPEED, JUMP_MAX_GRAVITY, SLIDE_SPEED_DECELERATION, SPRING_BOUNCE_DURATION,
    SPRING_BOUNCE_VELOCITY, SPRING_JUMP_DURATION, SPRING_JUMP_VELOCITY, SUBPIXELS,
    TARGET_WALK_SPEED, TOAST_HEIGHT, TOAST_SPEED, TOAST_TIME, VIEWPORT_PAN_SPEED,
    WALK_SPEED_ACCELERATION, WALK_SPEED_DECELERATION, WALL_JUMP_HORIZONTAL_SPEED,
    WALL_JUMP_VERTICAL_SPEED, WALL_SLIDE_SPEED, WALL_SLIDE_TIME, WALL_STICK_TIME,
};
use crate::door::Door;
use crate::imagemanager::ImageManager;
use crate::inputmanager::{BinaryInput, InputManager};
use crate::platform::{Bagel, Button, Conveyor, MovingPlatform, Platform, PlatformType, Spring};
use crate::player::{Player, PlayerState};
use crate::rendercontext::{RenderContext, RenderLayer};
use crate::scene::{Scene, SceneResult};
use crate::smallintset::SmallIntSet;
use crate::soundmanager::SoundManager;
use crate::star::Star;
use crate::switchstate::SwitchState;
use crate::tilemap::TileMap;
use crate::tileset::{TileIndex, TileProperties, TileSetProperties};
use crate::utils::{cmp_in_direction, Color, Direction, Point, Rect};

struct PlatformIntersectionResult {
    offset: i32,
    platforms: SmallIntSet<usize>,
}

// The results of trying to move.
struct TryMovePlayerResult {
    offset: i32,
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
    platforms: SmallIntSet<usize>,
    tile_ids: SmallIntSet<TileIndex>,
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
    stuck_in_wall: bool,
    crushed_by_platform: bool,
}

struct Level<'a> {
    name: String,
    map_path: PathBuf,
    map: TileMap<'a>,
    player: Player<'a>,

    wall_stick_counter: i32,
    wall_stick_facing_right: bool,
    wall_slide_counter: i32,

    coyote_counter: i32,
    jump_grace_counter: i32,
    spring_counter: i32,

    previous_map_offset: Option<Point>,
    toast_text: String,
    toast_position: i32,
    toast_counter: i32,

    // platforms, stars, and doors
    platforms: Vec<Platform<'a>>,
    stars: Vec<Star<'a>>,
    doors: Vec<Door<'a>>,

    star_count: i32,
    current_platform: Option<usize>,
    current_slopes: SmallIntSet<usize>,
    switches: SwitchState,
    current_switch_tiles: SmallIntSet<usize>,
    current_door: Option<usize>,

    previous_transition: String,
}

fn inc_player_x(player: &mut Player, offset: i32) {
    player.x += offset;
}

fn inc_player_y(player: &mut Player, offset: i32) {
    player.y += offset;
}

impl<'a> Level<'a> {
    fn new<'b, S: RenderTarget>(
        map_path: &Path,
        images: &'b ImageManager<'b>,
    ) -> Result<Level<'b>> {
        let wall_stick_counter = WALL_STICK_TIME;
        let wall_stick_facing_right = false;
        let wall_slide_counter = WALL_SLIDE_TIME;

        let coyote_counter = COYOTE_TIME;
        let jump_grace_counter = 0;
        let spring_counter = 0;

        let toast_position = -TOAST_HEIGHT;
        let toast_counter = TOAST_TIME;

        let name: String = map_path
            .file_stem()
            .and_then(|s| s.to_str())
            .context("invalid filename")?
            .to_string();
        let toast_text = name.clone();
        let previous_map_offset = None;
        let map = TileMap::from_file(map_path, images)?;
        let mut player = Player::new(images)?;
        player.x = 128;
        player.y = 129;

        let star_count = 0;
        let switches = SwitchState::new();
        let current_switch_tiles = SmallIntSet::new();
        let current_slopes = SmallIntSet::new();
        let current_platform = None;
        let current_door = None;

        let mut platforms: Vec<Platform> = Vec::new();
        let mut stars = Vec::new();
        let mut doors = Vec::new();

        for obj in map.objects.iter() {
            if obj.properties.platform {
                platforms.push(MovingPlatform::new(obj, map.tileset.clone())?);
            }
            if obj.properties.bagel {
                platforms.push(Bagel::new(obj, map.tileset.clone())?);
            }
            if obj.properties.convey.is_some() {
                platforms.push(Conveyor::new(obj, map.tileset.clone())?);
            }
            if obj.properties.spring {
                platforms.push(Spring::new(obj, map.tileset.clone(), images)?);
            }
            if obj.properties.button {
                platforms.push(Button::new(obj, map.tileset.clone(), images)?);
            }
            if obj.properties.door {
                doors.push(Door::new(obj, images)?);
            }
            if obj.properties.star {
                stars.push(Star::new(obj, map.tileset.clone())?);
            }
        }

        let map_path = map_path.to_owned();
        let previous_transition = "".to_owned();

        Ok(Level {
            name,
            map_path,
            map,
            player,
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

impl<'a> Level<'a> {
    /*
     * Movement.
     */

    fn update_player_trajectory_x(&mut self, inputs: &InputManager) {
        if matches!(self.player.state, PlayerState::Crouching) {
            if self.player.dx > 0 {
                self.player.dx = (self.player.dx - SLIDE_SPEED_DECELERATION).max(0);
            } else if self.player.dx < 0 {
                self.player.dx = (self.player.dx + SLIDE_SPEED_DECELERATION).min(0);
            }
            return;
        }

        // Apply controller input.
        let mut target_dx = 0;
        if inputs.is_on(BinaryInput::PlayerLeft) && !inputs.is_on(BinaryInput::PlayerRight) {
            target_dx = -1 * TARGET_WALK_SPEED;
        } else if inputs.is_on(BinaryInput::PlayerRight) && !inputs.is_on(BinaryInput::PlayerLeft) {
            target_dx = TARGET_WALK_SPEED;
        }

        // Change the velocity toward the target velocity.
        if self.player.dx > 0 {
            // We're facing right.
            if target_dx > self.player.dx {
                self.player.dx += WALK_SPEED_ACCELERATION;
                self.player.dx = self.player.dx.min(target_dx);
            }
            if target_dx < self.player.dx {
                self.player.dx -= WALK_SPEED_DECELERATION;
                self.player.dx = self.player.dx.max(target_dx);
            }
        } else if self.player.dx < 0 {
            // We're facing left.
            if target_dx > self.player.dx {
                self.player.dx += WALK_SPEED_DECELERATION;
                self.player.dx = self.player.dx.min(target_dx);
            }
            if target_dx < self.player.dx {
                self.player.dx -= WALK_SPEED_ACCELERATION;
                self.player.dx = self.player.dx.max(target_dx);
            }
        } else {
            // We're stopped.
            if target_dx > self.player.dx {
                self.player.dx += WALK_SPEED_ACCELERATION;
                self.player.dx = self.player.dx.min(target_dx);
            }
            if target_dx < self.player.dx {
                self.player.dx -= WALK_SPEED_ACCELERATION;
                self.player.dx = self.player.dx.max(target_dx);
            }
        }
    }

    fn update_player_trajectory_y(&mut self, inputs: &InputManager) {
        match self.player.state {
            PlayerState::Standing | PlayerState::Crouching => {
                // Fall at least one pixel so that we hit the ground again.
                self.player.dy = self.player.dy.max(1);
            }
            PlayerState::Jumping => {
                // Apply gravity.
                if self.player.dy < JUMP_MAX_GRAVITY {
                    self.player.dy += JUMP_ACCELERATION;
                }
                self.player.dy = self.player.dy.min(JUMP_MAX_GRAVITY);
            }
            PlayerState::Falling => {
                // Apply gravity.
                if self.player.dy < FALL_MAX_GRAVITY {
                    self.player.dy += FALL_ACCELERATION;
                }
                self.player.dy = self.player.dy.min(FALL_MAX_GRAVITY);
            }
            PlayerState::WallSliding => {
                // When you first grab the wall, don't start sliding for a while.
                if self.wall_slide_counter > 0 {
                    self.wall_slide_counter -= 1;
                    self.player.dy = 0;
                } else {
                    self.player.dy = WALL_SLIDE_SPEED;
                }
            }
            PlayerState::Stopped => {}
        }
    }

    fn find_platform_intersections(
        &self,
        player_rect: Rect,
        direction: Direction,
        is_backwards: bool,
    ) -> PlatformIntersectionResult {
        let mut result = PlatformIntersectionResult {
            offset: 0,
            platforms: SmallIntSet::new(),
        };
        for (i, platform) in self.platforms.iter().enumerate() {
            let distance = platform.try_move_to(player_rect, direction, is_backwards);
            if distance == 0 {
                continue;
            }

            let cmp = cmp_in_direction(distance, result.offset, direction);
            if cmp < 0 {
                result.offset = distance;
                result.platforms = SmallIntSet::new();
                result.platforms.insert(i);
            } else if cmp == 0 {
                result.platforms.insert(i);
            }
        }
        result
    }

    // Returns how far this player needs to move in direction to not intersect, in sub-pixels.
    fn try_move_player(&self, direction: Direction, is_backwards: bool) -> TryMovePlayerResult {
        let player_rect = self.player.get_target_bounds_rect(direction);

        let map_result = self
            .map
            .try_move_to(player_rect, direction, &self.switches, is_backwards);
        let platform_result =
            self.find_platform_intersections(player_rect, direction, is_backwards);

        if cmp_in_direction(platform_result.offset, map_result.hard_offset, direction) <= 0 {
            TryMovePlayerResult {
                offset: platform_result.offset,
                platforms: platform_result.platforms,
                tile_ids: SmallIntSet::new(),
            }
        } else {
            TryMovePlayerResult {
                offset: map_result.hard_offset,
                platforms: SmallIntSet::new(),
                tile_ids: map_result.tile_ids,
            }
        }
    }

    // Returns whether the first move hit a wall or platform.
    fn move_and_check(
        &mut self,
        forward: Direction,
        apply_offset: fn(&mut Player, i32) -> (),
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
                result.on_ground = move_result1.offset != 0;
                result.on_tile_ids = move_result1.tile_ids;
                result.on_platforms = move_result1.platforms;
            }
            Direction::Up => {
                // If we're traveling up, then if we hit something below, it's not the ground,
                // unless we're standing on a platform.
                if !matches!(self.player.state, PlayerState::Jumping)
                    && !matches!(self.player.state, PlayerState::Falling)
                {
                    result.on_ground = move_result2.offset != 0;
                }
                result.hit_ceiling = move_result1.offset != 0;
                result.on_tile_ids = move_result2.tile_ids;
                result.on_platforms = move_result2.platforms;
            }
            Direction::Left | Direction::Right => result.against_wall = move_result1.offset != 0,
            Direction::None => panic!("cannot move to in none direction"),
        }

        // See if we're crushed.
        if offset != 0 {
            let crush_check = self.try_move_player(forward, false);
            if crush_check.offset != 0 {
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

    fn move_player_x(&mut self, inputs: &InputManager) -> MovePlayerXResult {
        let mut dx = self.player.dx;
        if let Some(current_platform) = self.current_platform {
            dx += self.platforms[current_platform].dx();
        }
        self.player.x += dx;

        let (move_result, pushing) = if dx < 0 || (dx == 0 && !self.player.facing_right) {
            // Moving left.
            let move_result = self.move_and_check(Direction::Left, inc_player_x);
            let pushing = inputs.is_on(BinaryInput::PlayerLeft);
            (move_result, pushing)
        } else {
            // Moving right.
            let move_result = self.move_and_check(Direction::Right, inc_player_x);
            let pushing = inputs.is_on(BinaryInput::PlayerRight);
            (move_result, pushing)
        };

        let result = MovePlayerXResult {
            pushing_against_wall: pushing && move_result.against_wall,
            crushed_by_platform: move_result.crushed_by_platform,
            stuck_in_wall: move_result.stuck_in_wall,
        };

        // If you're against the wall, you're stopped.
        if result.pushing_against_wall {
            self.player.dx = 0;
        }

        result
    }

    fn get_slope_dy(&self) -> i32 {
        let mut slope_fall = 0;
        for slope_id in self.current_slopes.iter() {
            let slope = self
                .map
                .tileset
                .get_slope(*slope_id)
                .expect("must be valid");
            let left_y = slope.left_y;
            let right_y = slope.right_y;
            let mut fall: i32 = 0;
            if self.player.dx > 0 || (self.player.dx == 0 && self.player.facing_right) {
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

    fn move_player_y(&mut self, sounds: &SoundManager) -> MovePlayerYResult {
        let mut dy = self.player.dy;
        if let Some(current_platform) = self.current_platform {
            // This could be positive or negative.
            dy += self.platforms[current_platform].dy();
        }

        // If you're on a slope, make sure to fall at least the slope amount.
        if dy >= 0 {
            dy = dy.max(self.get_slope_dy());
        }

        self.player.y += dy;

        if dy <= 0 {
            // Moving up.
            let move_result = self.move_and_check(Direction::Up, inc_player_y);
            if move_result.hit_ceiling {
                self.player.dy = 0;
            }

            self.handle_slopes(&move_result.on_tile_ids);
            self.handle_current_platforms(&move_result.on_platforms);

            MovePlayerYResult {
                on_ground: move_result.on_ground,
                crushed_by_platform: move_result.crushed_by_platform,
                stuck_in_wall: move_result.stuck_in_wall,
                platforms: SmallIntSet::new(),
                tile_ids: SmallIntSet::new(),
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
                tile_ids: move_result.on_tile_ids,
                platforms: move_result.on_platforms,
                crushed_by_platform: move_result.crushed_by_platform,
                stuck_in_wall: move_result.stuck_in_wall,
            }
        }
    }

    fn handle_slopes(&mut self, tiles: &SmallIntSet<TileIndex>) {
        self.current_slopes.clear();
        for tile_id in tiles.iter() {
            if let Some(TileProperties { slope: true, .. }) =
                self.map.tileset.get_tile_properties(*tile_id)
            {
                self.current_slopes.insert(*tile_id);
            }
        }
    }

    fn handle_spikes(&mut self, tiles: &SmallIntSet<TileIndex>) {
        for tile_id in tiles.iter() {
            if let Some(TileProperties { deadly: true, .. }) =
                self.map.tileset.get_tile_properties(*tile_id)
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

    fn handle_switch_tiles(&mut self, tiles: &SmallIntSet<TileIndex>, sounds: &SoundManager) {
        let new_switch_tiles = SmallIntSet::new();
        let previous = mem::replace(&mut self.current_switch_tiles, new_switch_tiles);
        for t in tiles.iter() {
            let Some(TileProperties {
                switch: Some(switch),
                ..
            }) = self.map.tileset.get_tile_properties(*t)
            else {
                continue;
            };
            self.current_switch_tiles.insert(*t);
            if previous.contains(*t) {
                continue;
            }
            // sounds.play(Sound.CLICK);
            self.switches.apply_command(switch);
        }
    }

    fn update_player_movement(
        &mut self,
        inputs: &InputManager,
        sounds: &SoundManager,
    ) -> PlayerMovementResult {
        self.update_player_trajectory_x(inputs);
        self.update_player_trajectory_y(inputs);

        let x_result = self.move_player_x(inputs);
        let y_result = self.move_player_y(sounds);

        PlayerMovementResult {
            on_ground: y_result.on_ground,
            pushing_against_wall: x_result.pushing_against_wall,
            jump_down: inputs.is_on(BinaryInput::PlayerJumpDown),
            jump_triggered: inputs.is_on(BinaryInput::PlayerJumpTrigger),
            crouch_down: inputs.is_on(BinaryInput::PlayerCrouch),
            stuck_in_wall: x_result.stuck_in_wall || y_result.stuck_in_wall,
            crushed_by_platform: x_result.crushed_by_platform || y_result.crushed_by_platform,
        }
    }

    fn update_player_state(&mut self, movement: PlayerMovementResult) {
        if movement.on_ground {
            self.coyote_counter = COYOTE_TIME;
        } else {
            if self.coyote_counter > 0 {
                self.coyote_counter -= 1;
            }
        }

        if self.jump_grace_counter > 0 {
            self.jump_grace_counter -= 1;
        }

        if movement.crushed_by_platform {
            self.player.state = PlayerState::Stopped;
            self.player.is_dead = true;
        } else {
            match self.player.state {
                PlayerState::Crouching | PlayerState::WallSliding | PlayerState::Stopped => {}
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
                            self.player.dy = -1 * SPRING_JUMP_VELOCITY;
                        } else {
                            self.spring_counter = SPRING_BOUNCE_DURATION;
                            self.player.dy = -1 * SPRING_BOUNCE_VELOCITY;
                        }
                    } else if self.coyote_counter == 0 {
                        self.player.state = PlayerState::Falling;
                        self.player.dy = 0;
                        if let Some(current_platform) = self.current_platform {
                            self.player.dx = self.platforms[current_platform].dx();
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
                                self.player.dy = -1 * SPRING_JUMP_VELOCITY;
                            } else {
                                self.spring_counter = 0;
                                self.player.dy = -1 * JUMP_INITIAL_SPEED;
                            }
                            if let Some(current_platform) = self.current_platform {
                                self.player.dx += self.platforms[current_platform].dx();
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
                        self.player.dy = 0;
                    } else {
                        if movement.pushing_against_wall && self.player.dy >= 0 {
                            self.player.state = PlayerState::WallSliding;
                            self.wall_slide_counter = WALL_SLIDE_TIME;
                        }
                    }
                }
                PlayerState::Jumping => {
                    if movement.on_ground {
                        self.player.state = PlayerState::Standing;
                        self.player.dy = 0;
                    } else if self.player.dy >= 0 {
                        self.player.state = PlayerState::Falling;
                    } else {
                        if !movement.jump_down {
                            if self.spring_counter == 0 {
                                self.player.state = PlayerState::Falling;
                                self.player.dy = 0;
                            } else {
                                self.spring_counter -= 1;
                            }
                        }
                    }
                }
                PlayerState::WallSliding => {
                    if movement.jump_triggered {
                        self.player.state = PlayerState::Jumping;
                        self.player.dy = -1 * WALL_JUMP_VERTICAL_SPEED;
                        if self.player.facing_right {
                            self.player.dx = -1 * WALL_JUMP_HORIZONTAL_SPEED;
                        } else {
                            self.player.dx = WALL_JUMP_HORIZONTAL_SPEED;
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
                        self.player.dy = 0;
                    } else if !movement.crouch_down {
                        self.player.state = PlayerState::Standing;
                    }
                }
            }
        }
    }

    fn update(&mut self, inputs: &InputManager, sounds: &SoundManager) -> SceneResult {
        if inputs.is_on(BinaryInput::Cancel) {
            return SceneResult::Pop;
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
                stuck_in_wall: false,
                crushed_by_platform: false,
            },
            _ => self.update_player_movement(inputs, sounds),
        };

        let start_state: PlayerState = self.player.state.clone();
        self.update_player_state(movement);

        // Make sure you aren't stuck in a wall.
        let player_rect = self.player.get_target_bounds_rect(Direction::None);

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

        let old_stars = mem::replace(&mut self.stars, Vec::new());
        for star in old_stars.into_iter() {
            if star.intersects(player_rect) {
                //sounds.play(Sound.STAR);
                self.star_count += 1;
                self.toast_text = format!("STARS x {}", self.star_count);
                self.toast_counter = TOAST_TIME;
            } else {
                self.stars.push(star);
            }
        }

        if true {
            // TODO: Include slopes.
            let attribs = format!(
                "{:?}, idle={}, platform={:?}",
                movement, self.player.is_idle, self.current_platform,
            );
            let transition = format!("{:?} x {} -> {:?}", start_state, attribs, self.player.state);
            if transition != self.previous_transition {
                self.previous_transition = transition;
                println!("{}", self.previous_transition);
            }
        }

        if self.player.is_dead {
            return SceneResult::SwitchToKillScreen {
                path: self.map_path.clone(),
            };
        }

        if self.toast_counter == 0 {
            if self.toast_position > -TOAST_HEIGHT {
                self.toast_position -= TOAST_SPEED;
            }
        } else {
            self.toast_counter -= 1;
            if self.toast_position < 0 {
                self.toast_position += TOAST_SPEED;
            }
        }

        SceneResult::Continue
    }

    fn draw<'b>(&mut self, context: &'b mut RenderContext<'a>, images: &'a ImageManager)
    where
        'a: 'b,
    {
        let dest = context.logical_area();

        // Make sure the player is on the screen, and then center them if possible.
        let player_rect = self.player.get_target_bounds_rect(Direction::None);
        let (preferred_x, preferred_y) = self.map.get_preferred_view(player_rect);
        let player_x = self.player.x;
        let player_y = self.player.y;
        let mut player_draw_x = dest.w / 2;
        let mut player_draw_y = dest.h / 2;
        // Don't waste space on the sides of the screen beyond the map.
        if player_draw_x > player_x {
            player_draw_x = player_x;
        }
        // The map is drawn 4 pixels from the top of the screen.
        if player_draw_y > player_y + 4 {
            player_draw_y = player_y + 4;
        }
        let right_limit = dest.w - (self.map.width * self.map.tilewidth * SUBPIXELS);
        if player_draw_x < player_x + right_limit {
            player_draw_x = player_x + right_limit;
        }
        let bottom_limit = dest.h - (self.map.height * self.map.tileheight * SUBPIXELS);
        if player_draw_y < player_y + bottom_limit {
            player_draw_y = player_y + bottom_limit;
        }
        let mut map_offset = Point::new(player_draw_x - player_x, player_draw_y - player_y);

        if let Some(preferred_x) = preferred_x {
            map_offset = (-preferred_x, map_offset.y).into();
            player_draw_x = player_x + map_offset.x;
        }
        if let Some(preferred_y) = preferred_y {
            map_offset = (map_offset.x, -preferred_y).into();
            player_draw_y = player_y + map_offset.y;
        }

        // Don't let the viewport move too much in between frames.
        if let Some(prev) = self.previous_map_offset.as_ref() {
            if (map_offset.x - prev.x).abs() > VIEWPORT_PAN_SPEED {
                if prev.x < map_offset.x {
                    map_offset = (prev.x + VIEWPORT_PAN_SPEED, map_offset.y).into();
                } else if prev.x > map_offset.x {
                    map_offset = (prev.x - VIEWPORT_PAN_SPEED, map_offset.y).into();
                }
                player_draw_x = player_x + map_offset.x;
            }
            if (map_offset.y - prev.y).abs() > VIEWPORT_PAN_SPEED {
                if prev.y < map_offset.y {
                    map_offset = (map_offset.x, prev.y + VIEWPORT_PAN_SPEED).into()
                } else if prev.y > map_offset.y {
                    map_offset = (map_offset.x, prev.y - VIEWPORT_PAN_SPEED).into();
                }
                player_draw_y = player_y + map_offset.y;
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
            door.draw_background(context, RenderLayer::Player, map_offset, images);
        }
        for platform in self.platforms.iter() {
            platform.draw(context, RenderLayer::Player, map_offset);
        }
        for star in self.stars.iter() {
            star.draw(context, RenderLayer::Player, map_offset);
        }
        self.player.draw(
            context,
            RenderLayer::Player,
            (player_draw_x, player_draw_y).into(),
        );
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
        if top_bar_area.bottom() > 0 {
            context.fill_rect(top_bar_area, RenderLayer::Hud, top_bar_bgcolor);
            images.font().draw_string(
                context,
                RenderLayer::Hud,
                (
                    top_bar_area.x + 2 * SUBPIXELS,
                    top_bar_area.y + 2 * SUBPIXELS,
                )
                    .into(),
                &self.toast_text,
            );
        }

        // context.dark = self.map.is_dark;

        /*
        spotlight_pos = (
            player_draw_x + 12 * SUBPIXELS,
            player_draw_y + 12 * SUBPIXELS)
        spotlight_radius = 120.0 * SUBPIXELS
        context.add_light(spotlight_pos, spotlight_radius)
        */
    }
}

impl<'a> Scene for Level<'a> {
    fn update(
        &mut self,
        inputs: &crate::inputmanager::InputManager,
        sounds: crate::soundmanager::SoundManager,
    ) -> Result<SceneResult> {
        unimplemented!();
    }

    fn draw(&self, images: &ImageManager) {}
}
