use std::path::Path;

use anyhow::Result;

use crate::constants::{DOOR_CLOSING_FRAMES, DOOR_SPEED, DOOR_UNLOCKING_FRAMES, SUBPIXELS};
use crate::imagemanager::ImageManager;
use crate::rendercontext::{RenderContext, RenderLayer};
use crate::sprite::SpriteSheet;
use crate::tilemap::MapObject;
use crate::utils::{intersect, Point, Rect};

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

pub struct Door<'a> {
    x: i32,
    y: i32,
    sprite: SpriteSheet<'a>,
    pub destination: Option<String>,
    stars_needed: i32,
    stars_remaining: i32,
    pub active: bool,
    state: DoorState,
    frame: u32,
}

impl<'a> Door<'a> {
    pub fn new<'b, 'c>(obj: &MapObject, images: &'c ImageManager<'b>) -> Result<Door<'b>>
    where
        'b: 'c,
    {
        let sprite_path = obj
            .properties
            .sprite
            .clone()
            .unwrap_or_else(|| "assets/sprites/door.png".to_owned());

        let sprite = images.load_spritesheet(Path::new(&sprite_path), 32, 32)?;
        let x = obj.position.x * SUBPIXELS;
        let y = obj.position.y * SUBPIXELS;
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
            x,
            y,
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

    pub fn draw_background<'b, 'c>(
        &self,
        context: &'b mut RenderContext<'a>,
        layer: RenderLayer,
        offset: Point,
        images: &'c ImageManager<'a>,
    ) where
        'a: 'b,
        'a: 'c,
    {
        let x = self.x + offset.x();
        let y = self.y + offset.y();
        let dest = Rect {
            x,
            y,
            w: 32 * SUBPIXELS,
            h: 32 * SUBPIXELS,
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
            let pos = Point::new(x + 8 * SUBPIXELS, y + 12 * SUBPIXELS);
            images.font().draw_string(context, layer, pos, &s);
        }
    }

    pub fn draw_foreground<'b>(
        &self,
        context: &'b mut RenderContext<'a>,
        layer: RenderLayer,
        offset: Point,
    ) {
        let x = self.x + offset.x();
        let y = self.y + offset.y();
        let dest = Rect {
            x,
            y,
            w: 32 * SUBPIXELS,
            h: 32 * SUBPIXELS,
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

    pub fn is_inside(&self, player_rect: Rect) -> bool {
        let door_rect = Rect {
            x: self.x + 8 * SUBPIXELS,
            y: self.y,
            w: 24 * SUBPIXELS,
            h: 32 * SUBPIXELS,
        };
        intersect(player_rect, door_rect)
    }

    pub fn update(&mut self, player_rect: Rect, star_count: i32) {
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
