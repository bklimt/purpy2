use std::path::Path;

use anyhow::Result;
use num_traits::Zero;

use crate::{
    constants::{IDLE_TIME, PLAYER_FRAMES_PER_FRAME},
    geometry::{Pixels, Point, Rect, Subpixels},
    imagemanager::ImageLoader,
    rendercontext::{RenderContext, RenderLayer},
    sprite::{AnimationStateMachine, SpriteSheet},
    utils::Direction,
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

pub struct Player {
    pub position: Point<Subpixels>,
    pub delta: Point<Subpixels>,
    pub facing_right: bool,
    pub state: PlayerState,
    // 24x24 sprite sheet
    pub is_idle: bool,
    pub is_dead: bool,

    sprite: SpriteSheet,
    animation_state_machine: AnimationStateMachine,
    frame: u32,
    frames_to_next_frame: i32,
    idle_counter: i32,
}

impl Player {
    pub fn new(images: &mut dyn ImageLoader) -> Result<Player> {
        let sprite = images.load_spritesheet(
            Path::new("assets/sprites/skelly2.png"),
            Pixels::new(24),
            Pixels::new(24),
        )?;
        let animation_state_machine =
            AnimationStateMachine::from_file(Path::new("assets/sprites/skelly2_states.txt"))?;

        Ok(Player {
            position: Point::zero(),
            delta: Point::zero(),
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

    pub fn update_sprite(&mut self) -> Result<()> {
        self.facing_right = if self.delta.x < Subpixels::zero() {
            false
        } else if self.delta.x > Subpixels::zero() {
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
                    if !self.delta.x.is_zero() {
                        "RUNNING"
                    } else if self.idle_counter > 0 {
                        self.idle_counter -= 1;
                        "STANDING"
                    } else {
                        self.is_idle = true;
                        "IDLE"
                    }
                }
            }
        };

        if !matches!(self.state, PlayerState::Standing) || !self.delta.x.is_zero() {
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

    pub fn draw(&self, context: &mut RenderContext, layer: RenderLayer, pos: Point<Subpixels>) {
        let dest = Rect {
            x: pos.x,
            y: pos.y,
            w: Subpixels::from_pixels(24),
            h: Subpixels::from_pixels(24),
        };

        self.sprite
            .blit(context, layer, dest, self.frame, 0, !self.facing_right);
    }

    fn get_raw_target_bounds(&self, direction: Option<Direction>) -> Rect<Pixels> {
        let (x, y, w, h) = match self.state {
            PlayerState::Crouching => match direction {
                Some(Direction::Down) => (8, 19, 8, 4),
                _ => (8, 14, 8, 9),
            },
            _ => match direction {
                None => (8, 4, 8, 19),
                Some(Direction::Up) => (8, 4, 8, 4),
                Some(Direction::Down) => (8, 19, 8, 4),
                Some(Direction::Right) => (12, 4, 4, 14),
                Some(Direction::Left) => (8, 4, 4, 14),
            },
        };
        Rect {
            x: Pixels::new(x),
            y: Pixels::new(y),
            w: Pixels::new(w),
            h: Pixels::new(h),
        }
    }

    // Returns the bounds rect in subpixels to check when moving in direction.
    pub fn get_target_bounds_rect(&self, direction: Option<Direction>) -> Rect<Subpixels> {
        let raw_bounds: Rect<Subpixels> = self.get_raw_target_bounds(direction).into();
        raw_bounds + self.position
    }
}
