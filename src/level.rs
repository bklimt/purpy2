use crate::door::Door;
use crate::platform::Platform;
use crate::player::Player;
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
    // parent: Scene | None
    name: String,
    map: TileMap<'a>,
    player: Player<'a>,

    // restart_func: typing.Callable[[], Scene]
    /// next_func: typing.Callable[[str], Scene]
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
    platforms: Vec<Box<dyn Platform>>,
    stars: Vec<Star<'a>>,
    doors: Vec<Door<'a>>,

    current_platform: Option<usize>,
    current_slopes: SmallIntSet,
    switches: SwitchState,
    current_switch_tiles: SmallIntSet,
    current_door: Option<usize>,
}
