use std::path::Path;

use anyhow::Result;

use crate::{
    constants::{IDLE_TIME, PLAYER_FRAMES_PER_FRAME, SUBPIXELS},
    imagemanager::ImageManager,
    rendercontext::{RenderContext, RenderLayer},
    sprite::{AnimationStateMachine, SpriteSheet},
    utils::{Direction, Point, Rect},
};

#[derive(Debug, Clone, Copy)]
pub enum PlayerState {
    Falling,
    Standing,
    Crouching,
    WallSliding,
    Stopped,
    Jumping,
}

pub struct Player<'a> {
    pub x: i32,
    pub y: i32,
    pub dx: i32,
    pub dy: i32,
    pub facing_right: bool,
    pub state: PlayerState,
    // 24x24 sprite sheet
    pub is_idle: bool,
    pub is_dead: bool,

    sprite: SpriteSheet<'a>,
    animation_state_machine: AnimationStateMachine,
    frame: u32,
    frames_to_next_frame: i32,
    idle_counter: i32,
}

impl<'a> Player<'a> {
    pub fn new<'b>(images: &ImageManager<'b>) -> Result<Player<'b>> {
        let sprite = images.load_spritesheet(Path::new("assets/sprites/skelly2.png"), 24, 24)?;
        let animation_state_machine =
            AnimationStateMachine::from_file(Path::new("assets/sprites/skelly2_states.txt"))?;

        Ok(Player {
            x: 0,
            y: 0,
            dx: 0,
            dy: 0,
            facing_right: true,
            sprite,
            animation_state_machine,
            state: PlayerState::Standing,
            frame: 0,
            frames_to_next_frame: PLAYER_FRAMES_PER_FRAME,
            idle_counter: IDLE_TIME,
            is_idle: false,
            is_dead: false,
        })
    }

    fn update_sprite(&mut self) -> Result<()> {
        self.facing_right = if self.dx < 0 {
            false
        } else if self.dx > 0 {
            true
        } else {
            self.facing_right
        };

        let state = if self.is_dead {
            // TODO
            /*
            x_jiggle = randint(-SUBPIXELS, SUBPIXELS)
            y_jiggle = randint(-SUBPIXELS, SUBPIXELS)
            pos = (pos[0] + x_jiggle, pos[1] + y_jiggle)
            */
            "DEAD"
        } else {
            match self.state {
                PlayerState::Falling => "FALLING",
                PlayerState::Jumping => "JUMPING",
                PlayerState::WallSliding => "WALL_SLIDING",
                PlayerState::Crouching => "CROUCHING",
                PlayerState::Standing | PlayerState::Stopped => {
                    if self.idle_counter > 0 {
                        self.idle_counter -= 1;
                        "STANDING"
                    } else {
                        self.is_idle = true;
                        "IDLE"
                    }
                }
            }
        };

        if !matches!(self.state, PlayerState::Standing) || self.dx != 0 {
            self.is_idle = false;
            self.idle_counter = IDLE_TIME;
        }

        if self.frames_to_next_frame == 0 {
            self.frame = self
                .animation_state_machine
                .next_frame(self.frame + 1, state)?
                - 1;
            self.frames_to_next_frame = PLAYER_FRAMES_PER_FRAME;
        } else {
            self.frames_to_next_frame -= 1;
        }

        Ok(())
    }

    fn draw(&self, context: &'a mut RenderContext<'a>, layer: RenderLayer, pos: Point) {
        let dest = Rect {
            x: pos.x(),
            y: pos.y(),
            w: 24 * SUBPIXELS,
            h: 24 * SUBPIXELS,
        };

        self.sprite
            .blit(context, layer, dest, self.frame, 0, !self.facing_right);
    }

    fn get_raw_target_bounds(&self, direction: Direction) -> (i32, i32, i32, i32) {
        match self.state {
            PlayerState::Crouching => (8, 14, 8, 9),
            _ => match direction {
                Direction::None => (8, 4, 8, 19),
                Direction::Up => (8, 4, 8, 4),
                Direction::Down => (8, 19, 8, 4),
                Direction::Right => (12, 4, 4, 14),
                Direction::Left => (8, 4, 4, 14),
            },
        }
    }

    // Returns the bounds rect in subpixels to check when moving in direction.
    pub fn get_target_bounds_rect(&self, direction: Direction) -> Rect {
        let unscaled = self.get_raw_target_bounds(direction);
        Rect {
            x: self.x + unscaled.0 * SUBPIXELS,
            y: self.y + unscaled.1 * SUBPIXELS,
            w: unscaled.2 * SUBPIXELS,
            h: unscaled.3 * SUBPIXELS,
        }
    }
}
