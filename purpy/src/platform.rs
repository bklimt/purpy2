// Most of the platform subtypes have a "new" that returns a Platform.
#![allow(clippy::new_ret_no_self)]

use std::mem;
use std::path::Path;
use std::rc::Rc;

use anyhow::{Context, Result};
use num_traits::Zero;
use rand::random;

use crate::constants::{
    BAGEL_FALL_TIME, BAGEL_GRAVITY_ACCELERATION, BAGEL_MAX_GRAVITY, BAGEL_WAIT_TIME, BUTTON_DELAY,
    BUTTON_MAX_LEVEL, SPRING_SPEED, SPRING_STALL_FRAMES, SPRING_STEPS,
};
use crate::geometry::{Pixels, Point, Rect, Subpixels};
use crate::imagemanager::ImageLoader;
use crate::rendercontext::{RenderContext, RenderLayer};
use crate::soundmanager::{Sound, SoundManager};
use crate::sprite::SpriteSheet;
use crate::switchstate::SwitchState;
use crate::tilemap::TileIndex;
use crate::tilemap::{ButtonType, ConveyorDirection, MapObject, Overflow, TileMap};
use crate::utils::{try_move_to_bounds, Direction};

pub enum PlatformType {
    MovingPlatform(MovingPlatform),
    Bagel(Bagel),
    Conveyor(Conveyor),
    Spring(Spring),
    Button(Button),
}

pub struct Platform {
    _id: i32,
    tilemap: Rc<TileMap>,
    tile_gid: TileIndex,
    position: Rect<Subpixels>,
    delta: Point<Subpixels>,
    solid: bool,
    occupied: bool,
    pub subtype: PlatformType,
}

impl Platform {
    fn new(obj: &MapObject, tilemap: Rc<TileMap>, subtype: PlatformType) -> Result<Platform> {
        // TODO: This shouldn't compile.
        Ok(Platform {
            _id: obj.id,
            tilemap,
            tile_gid: obj.gid.context("gid required for platforms")?,
            position: obj.position.into(),
            delta: Point::zero(),
            solid: obj.properties.solid,
            occupied: false,
            subtype,
        })
    }

    pub fn update(&mut self, switches: &mut SwitchState, sounds: &mut SoundManager) {
        // Temporarily swap out the subtype so we can pass both it and self as &mut.
        // Conveyor is just a subtype that is trivial to construct.
        let mut subtype = mem::replace(&mut self.subtype, PlatformType::Conveyor(Conveyor(())));
        match &mut subtype {
            PlatformType::MovingPlatform(platform) => platform.update(self, switches, sounds),
            PlatformType::Bagel(bagel) => bagel.update(self, switches, sounds),
            PlatformType::Conveyor(_) => {}
            PlatformType::Spring(spring) => spring.update(self, switches, sounds),
            PlatformType::Button(button) => button.update(self, switches, sounds),
        }
        self.subtype = subtype;
    }

    pub fn draw(&self, context: &mut RenderContext, layer: RenderLayer, offset: Point<Subpixels>) {
        match &self.subtype {
            PlatformType::Bagel(bagel) => bagel.draw(self, context, layer, offset),
            PlatformType::Spring(spring) => spring.draw(self, context, layer, offset),
            PlatformType::Button(button) => button.draw(self, context, layer, offset),
            _ => {
                let x = self.position.x + offset.x;
                let y = self.position.y + offset.y;
                let dest = Rect {
                    x,
                    y,
                    w: self.position.w,
                    h: self.position.h,
                };
                if let Some(anim) = self.tilemap.get_animation(self.tile_gid) {
                    anim.blit(context, layer, dest, false);
                } else {
                    self.tilemap.draw_tile(context, self.tile_gid, layer, dest);
                }
            }
        }
    }

    pub fn try_move_to(
        &self,
        player_rect: Rect<Subpixels>,
        direction: Direction,
        is_backwards: bool,
    ) -> Subpixels {
        if let PlatformType::Spring(spring) = &self.subtype {
            return spring.try_move_to(self, player_rect, direction, is_backwards);
        }

        let area = if self.solid {
            self.position
        } else {
            if !matches!(direction, Direction::Down) {
                return Subpixels::zero();
            }
            if is_backwards {
                return Subpixels::zero();
            }
            Rect {
                x: self.position.x,
                y: self.position.y,
                w: self.position.w,
                h: self.position.h / 2,
            }
        };
        try_move_to_bounds(player_rect, area, direction)
    }

    pub fn is_solid(&self) -> bool {
        self.solid
    }
    pub fn dx(&self) -> Subpixels {
        self.delta.x
    }
    pub fn dy(&self) -> Subpixels {
        self.delta.y
    }
    pub fn set_occupied(&mut self, occupied: bool) {
        self.occupied = occupied;
    }
}

pub struct MovingPlatform {
    direction: Direction,
    distance: Subpixels,
    speed: Subpixels,
    start: Point<Subpixels>,
    end: Point<Subpixels>,
    moving_forward: bool,
    condition: Option<String>,
    overflow: Overflow,
}

impl MovingPlatform {
    pub fn new(obj: &MapObject, tilemap: Rc<TileMap>) -> Result<Platform> {
        let (dist_mult, sx, sy) = match obj.properties.direction {
            Direction::Up => (tilemap.tileheight, 0, -1),
            Direction::Down => (tilemap.tileheight, 0, 1),
            Direction::Left => (tilemap.tilewidth, -1, 0),
            Direction::Right => (tilemap.tilewidth, 1, 0),
        };

        // This is 16 for historical reasons, just because that's what the speed is tuned for.
        let speed = obj.properties.speed.unwrap_or(Pixels::new(1));
        let speed = speed.as_subpixels() / 16;
        let dist_mult = dist_mult.as_subpixels();
        let distance = dist_mult * obj.properties.distance;
        let direction = obj.properties.direction;
        let start = obj.position.top_left().into();
        let end = start + Point::new(distance * sx, distance * sy);
        let moving_forward = true;
        let condition = obj.properties.condition.clone();
        let overflow = obj.properties.overflow;

        let moving_platform = MovingPlatform {
            direction,
            speed,
            distance,
            start,
            end,
            moving_forward,
            condition,
            overflow,
        };
        Platform::new(obj, tilemap, PlatformType::MovingPlatform(moving_platform))
    }

    fn update(&mut self, base: &mut Platform, switches: &mut SwitchState, _sounds: &SoundManager) {
        if let Some(condition) = self.condition.as_deref() {
            if !switches.is_condition_true(condition) {
                self.moving_forward = false;
                if base.position.top_left() == self.start {
                    base.delta = Point::zero();
                    return;
                }
            }
        }

        base.delta.x = self.speed * (self.end.x - self.start.x).sign();
        base.delta.y = self.speed * (self.end.y - self.start.y).sign();
        if self.moving_forward {
            match self.direction {
                Direction::Up => {
                    if base.position.y <= self.end.y {
                        match self.overflow {
                            Overflow::Wrap => base.position.y += self.distance,
                            Overflow::Clamp => {
                                base.delta.y = Subpixels::zero();
                                base.position.y = self.end.y + Subpixels::new(1);
                            }
                            Overflow::Oscillate => {
                                base.delta.y *= -1;
                                self.moving_forward = false;
                            }
                        }
                    }
                }
                Direction::Down => {
                    if base.position.y >= self.end.y {
                        match self.overflow {
                            Overflow::Wrap => {
                                base.position.y = self.start.y + (self.end.y - base.position.y)
                            }

                            Overflow::Clamp => {
                                base.delta.y = Subpixels::zero();
                                base.position.y = self.end.y - Subpixels::new(1);
                            }
                            Overflow::Oscillate => {
                                base.delta.y *= -1;
                                self.moving_forward = false;
                            }
                        }
                    }
                }
                Direction::Left => {
                    if base.position.x <= self.end.x {
                        match self.overflow {
                            Overflow::Wrap => base.position.x += self.distance,

                            Overflow::Clamp => {
                                base.delta.x = Subpixels::zero();
                                base.position.x = self.end.x + Subpixels::new(1);
                            }
                            Overflow::Oscillate => {
                                base.delta.x *= -1;
                                self.moving_forward = false;
                            }
                        }
                    }
                }
                Direction::Right => {
                    if base.position.x >= self.end.x {
                        match self.overflow {
                            Overflow::Wrap => {
                                base.position.x = self.start.x + (self.end.x - base.position.x)
                            }

                            Overflow::Clamp => {
                                base.delta.x = Subpixels::zero();
                                base.position.x = self.end.x - Subpixels::new(1);
                            }
                            Overflow::Oscillate => {
                                base.delta.x *= -1;
                                self.moving_forward = false;
                            }
                        }
                    }
                }
            }
        } else {
            // If must be oscillating.
            let at_start = match self.direction {
                Direction::Up => base.position.y >= self.start.y,
                Direction::Down => base.position.y <= self.start.y,
                Direction::Left => base.position.x >= self.start.x,
                Direction::Right => base.position.x <= self.start.x,
            };
            if at_start {
                self.moving_forward = true;
            } else {
                base.delta.x *= -1;
                base.delta.y *= -1;
            }
        }
        base.position += base.delta;
    }
}

pub struct Bagel {
    original_y: Subpixels,
    falling: bool,
    remaining: i32,
}

impl Bagel {
    pub fn new(obj: &MapObject, tilemap: Rc<TileMap>) -> Result<Platform> {
        let original_y = obj.position.y.as_subpixels();
        let bagel = Bagel {
            original_y,
            falling: false,
            remaining: BAGEL_WAIT_TIME,
        };
        Platform::new(obj, tilemap, PlatformType::Bagel(bagel))
    }

    fn draw(
        &self,
        base: &Platform,
        context: &mut RenderContext,
        layer: RenderLayer,
        offset: Point<Subpixels>,
    ) {
        let mut x = base.position.x + offset.x;
        let mut y = base.position.y + offset.y;
        if base.occupied {
            x += Subpixels::new((random::<u8>() % 3) as i32 - 1);
            y += Subpixels::new((random::<u8>() % 3) as i32 - 1);
        }
        let dest = Rect {
            x,
            y,
            w: base.tilemap.tilewidth.as_subpixels(),
            h: base.tilemap.tileheight.as_subpixels(),
        };
        base.tilemap.draw_tile(context, base.tile_gid, layer, dest);
    }

    fn update(&mut self, base: &mut Platform, _switches: &mut SwitchState, _sounds: &SoundManager) {
        if self.falling {
            self.remaining -= 1;
            if self.remaining == 0 {
                base.delta.y = Subpixels::zero();
                base.position.y = self.original_y;
                self.falling = false;
                self.remaining = BAGEL_WAIT_TIME;
            } else {
                base.delta.y += BAGEL_GRAVITY_ACCELERATION;
                base.delta.y = base.delta.y.max(BAGEL_MAX_GRAVITY);
                base.position.y += base.delta.y;
            }
        } else if base.occupied {
            self.remaining -= 1;
            if self.remaining == 0 {
                self.falling = true;
                self.remaining = BAGEL_FALL_TIME;
                base.delta.y = Subpixels::zero();
            }
        } else {
            self.remaining = BAGEL_WAIT_TIME;
        }
    }
}

pub struct Conveyor(());

impl Conveyor {
    pub fn new(obj: &MapObject, tilemap: Rc<TileMap>) -> Result<Platform> {
        // This is hand-tuned.
        let speed = (obj
            .properties
            .speed
            .unwrap_or(Pixels::new(24))
            .as_subpixels())
            / 16;
        let dx = match obj
            .properties
            .convey
            .expect("conveyor does not have convey property")
        {
            ConveyorDirection::Left => speed * -1,
            ConveyorDirection::Right => speed,
        };

        let mut base = Platform::new(obj, tilemap, PlatformType::Conveyor(Conveyor(())))?;
        base.delta.x = dx;
        Ok(base)
    }
}

pub struct Spring {
    sprite: SpriteSheet,
    up: bool,
    pos: Subpixels,
    stall_counter: i32,
    pub launch: bool,
}

impl Spring {
    pub fn new(
        obj: &MapObject,
        tileset: Rc<TileMap>,
        images: &mut dyn ImageLoader,
    ) -> Result<Platform> {
        let path = Path::new("assets/sprites/spring.png");
        let sprite = images.load_spritesheet(path, Pixels::new(8), Pixels::new(8))?;
        let spring = Spring {
            sprite,
            up: false,
            pos: Subpixels::zero(),
            stall_counter: SPRING_STALL_FRAMES,
            launch: false,
        };
        Platform::new(obj, tileset, PlatformType::Spring(spring))
    }

    fn frame(&self) -> i32 {
        self.pos.as_pixels() / Pixels::new(1)
    }

    pub fn should_boost(&self) -> bool {
        self.up || (self.frame() == SPRING_STEPS - 1)
    }

    fn draw(
        &self,
        base: &Platform,
        context: &mut RenderContext,
        layer: RenderLayer,
        offset: Point<Subpixels>,
    ) {
        let x = base.position.x + offset.x;
        let y = base.position.y + offset.y;
        let dest = Rect {
            x,
            y,
            w: base.position.w,
            h: base.position.h,
        };
        self.sprite
            .blit(context, layer, dest, self.frame() as u32, 0, false);
    }

    fn update(&mut self, base: &mut Platform, _switches: &mut SwitchState, _sounds: &SoundManager) {
        base.delta = Point::zero();
        self.launch = false;
        if !base.occupied {
            // You're not on it, so it resets.
            self.stall_counter = SPRING_STALL_FRAMES;
            self.up = false;
            if self.pos > Subpixels::zero() {
                self.pos -= SPRING_SPEED;
                base.delta.y = SPRING_SPEED * -1;
            }
        } else if self.up {
            // It's currently bouncing up.
            self.stall_counter = SPRING_STALL_FRAMES;
            if self.pos > Subpixels::zero() {
                self.pos -= SPRING_SPEED;
                base.delta.y = SPRING_SPEED * -1;
            } else {
                self.launch = true;
            }
        } else if self.pos < (Pixels::new(SPRING_STEPS).as_subpixels()) - SPRING_SPEED {
            // It's still moving down.
            self.stall_counter = SPRING_STALL_FRAMES;
            self.pos += SPRING_SPEED;
            base.delta.y = SPRING_SPEED;
        } else if self.stall_counter > 0 {
            // It's reached the bottom, but hasn't been there long enough.
            self.stall_counter -= 1;
        } else {
            // It's been at the bottom long enough, it's time to move up.
            self.stall_counter = SPRING_STALL_FRAMES;
            self.up = true;
        }
    }

    fn try_move_to(
        &self,
        base: &Platform,
        player_rect: Rect<Subpixels>,
        direction: Direction,
        is_backwards: bool,
    ) -> Subpixels {
        if base.solid {
            let area = Rect {
                x: base.position.x,
                y: base.position.y + self.pos,
                w: base.position.w,
                h: base.position.h - self.pos,
            };
            try_move_to_bounds(player_rect, area, direction)
        } else {
            if !matches!(direction, Direction::Down) {
                return Subpixels::zero();
            }
            if is_backwards {
                return Subpixels::zero();
            }
            let area = Rect {
                x: base.position.x,
                y: base.position.y + self.pos,
                w: base.position.w,
                h: base.position.h / 2,
            };
            try_move_to_bounds(player_rect, area, direction)
        }
    }
}

pub struct Button {
    sprite: SpriteSheet,
    level: u32,
    original_y: Subpixels,
    clicked: bool,
    button_type: ButtonType,
    was_occupied: bool,
    color: String,
}

fn get_button_image_path(color: &str) -> String {
    let color = if color == "!white" { "black" } else { color };
    format!("assets/sprites/buttons/{color}.png")
}

impl Button {
    pub fn new(
        obj: &MapObject,
        tileset: Rc<TileMap>,
        images: &mut dyn ImageLoader,
    ) -> Result<Platform> {
        let level = 0;
        let clicked = false;
        let was_occupied = false;

        let color = obj
            .properties
            .color
            .clone()
            .unwrap_or_else(|| "red".to_string());
        let image_path = get_button_image_path(&color);
        let sprite =
            images.load_spritesheet(Path::new(&image_path), Pixels::new(8), Pixels::new(8))?;
        let button_type = obj.properties.button_type;

        let original_y = obj.position.y.as_subpixels();
        let button = Button {
            sprite,
            level,
            original_y,
            clicked,
            button_type,
            was_occupied,
            color,
        };

        let mut base = Platform::new(obj, tileset, PlatformType::Button(button))?;
        // Move down by a whole pixel while on a button.
        base.delta.y = Pixels::new(1).as_subpixels();
        Ok(base)
    }

    fn draw(
        &self,
        base: &Platform,
        context: &mut RenderContext,
        layer: RenderLayer,
        offset: Point<Subpixels>,
    ) {
        let x = base.position.x + offset.x;
        let y = self.original_y + offset.y;
        let dest = Rect {
            x,
            y,
            w: base.position.w,
            h: base.position.h,
        };
        self.sprite
            .blit(context, layer, dest, self.level / BUTTON_DELAY, 0, false);
    }

    fn update(
        &mut self,
        base: &mut Platform,
        switches: &mut SwitchState,
        sounds: &mut SoundManager,
    ) {
        let was_clicked = self.clicked;

        if matches!(self.button_type, ButtonType::Smart) {
            self.clicked = switches.is_condition_true(&self.color);
        }

        if base.occupied && !self.was_occupied {
            self.clicked = match self.button_type {
                ButtonType::OneShot | ButtonType::Smart => true,
                ButtonType::Toggle => !self.clicked,
                _ => self.clicked,
            };
        }

        self.was_occupied = base.occupied;

        if matches!(self.button_type, ButtonType::Momentary) {
            self.clicked = base.occupied;
        }

        if self.clicked {
            if self.level < BUTTON_MAX_LEVEL {
                self.level += 1;
            }
        } else if self.level > 0 {
            self.level -= 1;
        }

        base.position.y =
            self.original_y + (Pixels::new(self.level as i32).as_subpixels() / BUTTON_DELAY as i32);

        if self.clicked != was_clicked {
            sounds.play(Sound::Click);
            if matches!(self.button_type, ButtonType::Smart) {
                if self.clicked && base.occupied {
                    switches.apply_command(&self.color);
                }
            } else if self.clicked || !matches!(self.button_type, ButtonType::OneShot) {
                switches.toggle(&self.color);
            }
        }
    }
}
