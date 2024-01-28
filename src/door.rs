use std::path::Path;

use anyhow::Result;

use crate::constants::{DOOR_CLOSING_FRAMES, DOOR_SPEED, DOOR_UNLOCKING_FRAMES};
use crate::font::Font;
use crate::geometry::{Pixels, Point, Rect, Subpixels};
use crate::imagemanager::ImageLoader;
use crate::rendercontext::{RenderContext, RenderLayer};
use crate::sprite::SpriteSheet;
use crate::tilemap::MapObject;
use crate::utils::intersect;

enum DoorLayer {
    Inactive = 0,
    Active,
    Locked,
    Doors,
    Frame,
}

enum DoorState {
    Locked = 1,
    Unlocking,
    Open,
    Closing,
    Closed,
}

pub struct Door {
    position: Point<Subpixels>,
    sprite: SpriteSheet,
    pub destination: Option<String>,
    stars_needed: i32,
    stars_remaining: i32,
    pub active: bool,
    state: DoorState,
    frame: u32,
}

impl Door {
    pub fn new(obj: &MapObject, images: &mut dyn ImageLoader) -> Result<Door> {
        let sprite_path = obj
            .properties
            .sprite
            .clone()
            .unwrap_or_else(|| "assets/sprites/door.png".to_owned());

        let sprite =
            images.load_spritesheet(Path::new(&sprite_path), Pixels::new(32), Pixels::new(32))?;
        let position = obj.position.top_left().into();
        let active = false;
        let destination = obj.properties.destination.clone();
        let stars_needed = obj.properties.stars_needed;
        let stars_remaining = stars_needed;
        let state = if stars_needed > 0 {
            DoorState::Locked
        } else {
            DoorState::Open
        };
        let frame = 0;

        Ok(Door {
            position,
            sprite,
            destination,
            stars_needed,
            stars_remaining,
            active,
            state,
            frame,
        })
    }

    pub fn is_open(&self) -> bool {
        matches!(self.state, DoorState::Open)
    }

    pub fn is_closed(&self) -> bool {
        matches!(self.state, DoorState::Closed)
    }

    pub fn unlock(&mut self) {
        if !matches!(self.state, DoorState::Locked) {
            return;
        }
        self.state = DoorState::Unlocking;
        self.frame = 0;
    }

    pub fn close(&mut self) {
        if !matches!(self.state, DoorState::Open) {
            return;
        }
        self.state = DoorState::Closing;
        self.frame = 0;
    }

    pub fn draw_background(
        &self,
        context: &mut RenderContext,
        layer: RenderLayer,
        offset: Point<Subpixels>,
        font: &Font,
    ) {
        let pos = self.position + offset;
        let dest = Rect {
            x: pos.x,
            y: pos.y,
            w: Pixels::new(32).into(),
            h: Pixels::new(32).into(),
        };
        let door_layer = if self.active {
            DoorLayer::Active
        } else {
            DoorLayer::Inactive
        } as u32;
        self.sprite.blit(context, layer, dest, 0, door_layer, false);
        if let Some(locked_index) = match self.state {
            DoorState::Locked => Some(0),
            DoorState::Unlocking => Some(self.frame / DOOR_SPEED),
            _ => None,
        } {
            self.sprite.blit(
                context,
                layer,
                dest,
                locked_index,
                DoorLayer::Locked as u32,
                false,
            );
        }
        if self.stars_remaining > 0 {
            let s = format!("{:02}", self.stars_remaining);
            let inset = Point::new(Pixels::new(8).into(), Pixels::new(12).into());
            let pos = pos + inset;
            font.draw_string(context, layer, pos, &s);
        }
    }

    pub fn draw_foreground(
        &self,
        context: &mut RenderContext,
        layer: RenderLayer,
        offset: Point<Subpixels>,
    ) {
        let pos = self.position + offset;
        let dest = Rect {
            x: pos.x,
            y: pos.y,
            w: Pixels::new(32).into(),
            h: Pixels::new(32).into(),
        };
        if let Some(door_index) = match self.state {
            DoorState::Closing => Some(self.frame / DOOR_SPEED),
            DoorState::Closed => Some(DOOR_CLOSING_FRAMES - 1),
            _ => None,
        } {
            self.sprite.blit(
                context,
                layer,
                dest,
                door_index,
                DoorLayer::Doors as u32,
                false,
            );
        }
        self.sprite
            .blit(context, layer, dest, 0, DoorLayer::Frame as u32, false);
    }

    pub fn is_inside(&self, player_rect: Rect<Subpixels>) -> bool {
        let inner: Rect<Subpixels> = Rect {
            x: Pixels::new(8),
            y: Pixels::new(0),
            w: Pixels::new(24),
            h: Pixels::new(32),
        }
        .into();
        let door_rect = inner + self.position;
        intersect(player_rect, door_rect)
    }

    pub fn update(&mut self, player_rect: Rect<Subpixels>, star_count: i32) {
        self.active = self.is_inside(player_rect);
        self.stars_remaining = (self.stars_needed - star_count).max(0);

        match self.state {
            DoorState::Unlocking => {
                let max_frame = DOOR_UNLOCKING_FRAMES * DOOR_SPEED;
                if self.frame == max_frame {
                    self.state = DoorState::Open;
                }
                self.frame = (self.frame + 1).min(max_frame);
            }
            DoorState::Closing => {
                let max_frame = DOOR_CLOSING_FRAMES * DOOR_SPEED;
                if self.frame == max_frame {
                    self.state = DoorState::Closed;
                }
                self.frame = (self.frame + 1).min(max_frame);
            }
            DoorState::Locked => {
                if star_count >= self.stars_needed {
                    self.unlock();
                }
            }
            _ => {}
        }
    }
}
