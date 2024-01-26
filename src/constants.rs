use crate::utils::Subpixels;

// Basic window and render size.
pub const WINDOW_WIDTH: u32 = 1600;
pub const WINDOW_HEIGHT: u32 = 900;
pub const RENDER_WIDTH: u32 = 320;
pub const RENDER_HEIGHT: u32 = 180;
pub const FRAME_RATE: u32 = 60;

// How many subpixels to use for game logic.
pub const SUBPIXELS: Subpixels = 32;

// Rendering details.
// pub const MAX_LIGHTS: u32 = 20;

// How quickly should the viewport pan to where it wants to be.
pub const VIEWPORT_PAN_SPEED: i32 = 5 * SUBPIXELS;

// Horizontal speed.
pub const TARGET_WALK_SPEED: i32 = 32 * SUBPIXELS / 16;
pub const WALK_SPEED_ACCELERATION: i32 = 1 * SUBPIXELS / 16;
pub const WALK_SPEED_DECELERATION: i32 = 3 * SUBPIXELS / 16;
pub const SLIDE_SPEED_DECELERATION: i32 = 1;

// Vertical speed.
pub const COYOTE_TIME: i32 = 6; // How long to hover in the air before officially falling.
pub const JUMP_GRACE_TIME: i32 = 12; // How long to remember jump was pressed while falling.
pub const JUMP_INITIAL_SPEED: i32 = 48 * SUBPIXELS / 16;
pub const JUMP_ACCELERATION: i32 = 2 * SUBPIXELS / 16;
pub const JUMP_MAX_GRAVITY: i32 = 32 * SUBPIXELS / 16;
pub const FALL_ACCELERATION: i32 = 5 * SUBPIXELS / 16;
pub const FALL_MAX_GRAVITY: i32 = 32 * SUBPIXELS / 16;

// Wall sliding.
pub const WALL_SLIDE_SPEED: i32 = 4 * SUBPIXELS / 16;
pub const WALL_JUMP_HORIZONTAL_SPEED: i32 = 48 * SUBPIXELS / 16;
pub const WALL_JUMP_VERTICAL_SPEED: i32 = 48 * SUBPIXELS / 16;
pub const WALL_STICK_TIME: i32 = 3;
pub const WALL_SLIDE_TIME: i32 = 60;

// Player appearance.
pub const IDLE_TIME: i32 = 240; // How long before showing idle animation.
pub const PLAYER_FRAMES_PER_FRAME: i32 = 4; // How fast to animate the player.

// How the "toast" text pops up at the top of the screen.
pub const TOAST_TIME: i32 = 150;
pub const TOAST_HEIGHT: i32 = 12 * SUBPIXELS;
pub const TOAST_SPEED: i32 = 8 * SUBPIXELS / 16;

// Button switches.
pub const BUTTON_DELAY: u32 = 2; // How slowly the button goes down.
pub const BUTTON_MAX_LEVEL: u32 = BUTTON_DELAY * 3; // There are 4 frames of animation.

// Falling platforms that look like bagels.
pub const BAGEL_WAIT_TIME: i32 = 30;
pub const BAGEL_FALL_TIME: i32 = 150;
pub const BAGEL_MAX_GRAVITY: i32 = 11 * SUBPIXELS / 16;
pub const BAGEL_GRAVITY_ACCELERATION: i32 = 1;

// Springs that bounce you.
pub const SPRING_STEPS: i32 = 4; // This should match the spring animation.
pub const SPRING_STALL_FRAMES: i32 = 10; // How long the spring stays at the bottom.
pub const SPRING_SPEED: i32 = 16 * SUBPIXELS / 16; // How fast the spring itself moves.
pub const SPRING_BOUNCE_DURATION: i32 = 30; // How long to jump when bouncing.
pub const SPRING_BOUNCE_VELOCITY: i32 = JUMP_INITIAL_SPEED;
pub const SPRING_JUMP_DURATION: i32 = 10; // How long to jump when jumping from spring.
pub const SPRING_JUMP_VELOCITY: i32 = 78 * SUBPIXELS / 16;

// Doors.
pub const DOOR_SPEED: u32 = 3;
pub const DOOR_CLOSING_FRAMES: u32 = 9; // The should match the door animation frames.
pub const DOOR_UNLOCKING_FRAMES: u32 = 9;
