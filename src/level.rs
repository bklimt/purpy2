use std::path::Path;

use anyhow::{Context, Result};
use sdl2::render::RenderTarget;

use crate::constants::{COYOTE_TIME, TOAST_HEIGHT, TOAST_TIME, WALL_SLIDE_TIME, WALL_STICK_TIME};
use crate::door::Door;
use crate::imagemanager::ImageManager;
use crate::platform::{Bagel, Button, Conveyor, MovingPlatform, Platform, Spring};
use crate::player::Player;
use crate::scene::{Scene, SceneResult};
use crate::smallintset::SmallIntSet;
use crate::star::Star;
use crate::switchstate::SwitchState;
use crate::tilemap::TileMap;

struct PlatformIntersectionResult {
    offset: i32,
    platforms: SmallIntSet,
}

// The results of trying to move.
struct TryMovePlayerResult {
    offset: i32,
    tile_ids: SmallIntSet,
    platforms: SmallIntSet,
}

struct MoveAndCheckResult {
    on_ground: bool,
    on_tile_ids: SmallIntSet,
    on_platforms: SmallIntSet,
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
    platforms: SmallIntSet,
    tile_ids: SmallIntSet,
    stuck_in_wall: bool,
    crushed_by_platform: bool,
}

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
    map: TileMap<'a>,
    player: Player<'a>,

    wall_stick_counter: i32,
    wall_stick_facing_right: bool,
    wall_slide_counter: i32,

    coyote_counter: i32,
    jump_grace_counter: i32,
    spring_counter: i32,

    previous_map_offset: Option<(i32, i32)>,
    toast_text: String,
    toast_position: i32,
    toast_counter: i32,

    // platforms, stars, and doors
    platforms: Vec<Box<dyn Platform<'a> + 'a>>,
    stars: Vec<Star<'a>>,
    doors: Vec<Door<'a>>,

    star_count: i32,
    current_platform: Option<usize>,
    current_slopes: SmallIntSet,
    switches: SwitchState,
    current_switch_tiles: SmallIntSet,
    current_door: Option<usize>,
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

        let mut platforms: Vec<Box<dyn Platform>> = Vec::new();
        let mut stars = Vec::new();
        let mut doors = Vec::new();

        for obj in map.objects.iter() {
            if obj.properties.platform {
                platforms.push(Box::new(MovingPlatform::new(obj, map.tileset.clone())?));
            }
            if obj.properties.bagel {
                platforms.push(Box::new(Bagel::new(obj, map.tileset.clone())?));
            }
            if obj.properties.convey.is_some() {
                platforms.push(Box::new(Conveyor::new(obj, map.tileset.clone())?));
            }
            if obj.properties.spring {
                platforms.push(Box::new(Spring::new(obj, map.tileset.clone(), images)?));
            }
            if obj.properties.button {
                platforms.push(Box::new(Button::new(obj, map.tileset.clone(), images)?));
            }
            if obj.properties.door {
                doors.push(Door::new(obj, images)?);
            }
            if obj.properties.star {
                stars.push(Star::new(obj, map.tileset.clone())?);
            }
        }

        Ok(Level {
            name,
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
        })
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
