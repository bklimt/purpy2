use std::path::Path;

use anyhow::{Context, Result};
use rand::random;

use crate::constants::{
    BAGEL_FALL_TIME, BAGEL_GRAVITY_ACCELERATION, BAGEL_MAX_GRAVITY, BAGEL_WAIT_TIME, BUTTON_DELAY,
    BUTTON_MAX_LEVEL, SPRING_SPEED, SPRING_STALL_FRAMES, SPRING_STEPS, SUBPIXELS,
};
use crate::imagemanager::ImageManager;
use crate::soundmanager::SoundManager;
use crate::sprite::{SpriteBatch, SpriteSheet};
use crate::switchstate::SwitchState;
use crate::tilemap::{ButtonType, ConveyorDirection, MapObject, Overflow};
use crate::tileset::{TileIndex, TileSet};
use crate::utils::{sign, try_move_to_bounds, Direction, Point, Rect};

pub trait Platform {
    fn update(&mut self, switches: &mut SwitchState, sounds: &SoundManager);
    fn draw(&self, batch: &mut SpriteBatch, offset: Point);
    fn try_move_to(&self, player_rect: Rect, direction: Direction, is_backwards: bool) -> i32;
}

struct PlatformBase<'a> {
    id: i32,
    tileset: &'a TileSet<'a>,
    tile_id: TileIndex,
    position: Rect,
    dx: i32,
    dy: i32,
    solid: bool,
    occupied: bool,
}

impl<'a> PlatformBase<'a> {
    pub fn new<'b>(obj: MapObject, tileset: &'b TileSet<'b>) -> Result<PlatformBase<'b>> {
        Ok(PlatformBase {
            id: obj.id,
            tileset: tileset,
            tile_id: obj.gid.context("gid required for platforms")? as TileIndex - 1,
            position: obj.position,
            dx: 0,
            dy: 0,
            solid: obj.properties.solid,
            occupied: false,
        })
    }

    fn draw(&self, batch: &mut SpriteBatch, offset: Point) {
        let x = self.position.x + offset.x();
        let y = self.position.y + offset.y();
        let dest = self.position;
        if let Some(anim) = self.tileset.animations.get(&self.tile_id) {
            anim.blit(batch, dest, false);
        } else {
            let src = self.tileset.get_source_rect(self.tile_id);
            batch.draw(&self.tileset.sprite, Some(dest), Some(src));
        }
    }

    fn try_move_to(&self, player_rect: Rect, direction: Direction, is_backwards: bool) -> i32 {
        let area = if self.solid {
            self.position
        } else {
            if !matches!(direction, Direction::Down) {
                return 0;
            }
            if is_backwards {
                return 0;
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
}

pub struct MovingPlatform<'a> {
    base: PlatformBase<'a>,
    direction: Direction,
    distance: i32,
    speed: i32,
    start_x: i32,
    start_y: i32,
    end_x: i32,
    end_y: i32,
    moving_forward: bool,
    condition: Option<String>,
    overflow: Overflow,
}

impl<'a> MovingPlatform<'a> {
    fn new<'b>(mut obj: MapObject, tileset: &'b TileSet<'b>) -> Result<MovingPlatform<'b>> {
        let (dist_mult, dx, dy) = match obj.properties.direction {
            Direction::Up => (tileset.tileheight, 0, -obj.properties.distance),
            Direction::Down => (tileset.tileheight, 0, obj.properties.distance),
            Direction::Left => (tileset.tilewidth, -obj.properties.distance, 0),
            Direction::Right => (tileset.tilewidth, obj.properties.distance, 0),
            Direction::None => (0, 0, 0),
        };

        // This is 16 for historical reasons, just because that's what the speed is tuned for.
        let speed = obj.properties.speed.unwrap_or(1);
        let speed = (speed * SUBPIXELS) / 16;
        let dist_mult = dist_mult * SUBPIXELS;
        let distance = obj.properties.distance * dist_mult;
        let direction = obj.properties.direction;
        let start_x = obj.position.x;
        let start_y = obj.position.y;
        let end_x = start_x + dx;
        let end_y = start_y + dy;
        let moving_forward = true;
        let condition = obj.properties.condition.take();
        let overflow = obj.properties.overflow;

        Ok(MovingPlatform {
            base: PlatformBase::new(obj, tileset)?,
            direction,
            speed,
            distance,
            start_x,
            start_y,
            end_x,
            end_y,
            moving_forward,
            condition,
            overflow,
        })
    }
}

impl<'a> Platform for MovingPlatform<'a> {
    fn update(&mut self, switches: &mut SwitchState, sounds: &SoundManager) {
        if let Some(condition) = self.condition.as_deref() {
            if !switches.is_condition_true(&condition) {
                self.moving_forward = false;
                if self.base.position.x == self.start_x && self.base.position.y == self.start_y {
                    self.base.dx = 0;
                    self.base.dy = 0;
                    return;
                }
            }
        }

        self.base.dx = sign(self.end_x - self.start_x) * self.speed;
        self.base.dy = sign(self.end_y - self.start_y) * self.speed;
        if self.moving_forward {
            match self.direction {
                Direction::Up => {
                    if self.base.position.y <= self.end_y {
                        match self.overflow {
                            Overflow::Wrap => self.base.position.y += self.distance,
                            Overflow::Clamp => {
                                self.base.dy = 0;
                                self.base.position.y = self.end_y + 1;
                            }
                            Overflow::Oscillate => {
                                self.base.dy *= -1;
                                self.moving_forward = false;
                            }
                        }
                    }
                }
                Direction::Down => {
                    if self.base.position.y >= self.end_y {
                        match self.overflow {
                            Overflow::Wrap => {
                                self.base.position.y =
                                    self.start_y + (self.end_y - self.base.position.y)
                            }

                            Overflow::Clamp => {
                                self.base.dy = 0;
                                self.base.position.y = self.end_y - 1;
                            }
                            Overflow::Oscillate => {
                                self.base.dy *= -1;
                                self.moving_forward = false;
                            }
                        }
                    }
                }
                Direction::Left => {
                    if self.base.position.x <= self.end_x {
                        match self.overflow {
                            Overflow::Wrap => self.base.position.x += self.distance,

                            Overflow::Clamp => {
                                self.base.dx = 0;
                                self.base.position.x = self.end_x + 1;
                            }
                            Overflow::Oscillate => {
                                self.base.dx *= -1;
                                self.moving_forward = false;
                            }
                        }
                    }
                }
                Direction::Right => {
                    if self.base.position.x >= self.end_x {
                        match self.overflow {
                            Overflow::Wrap => {
                                self.base.position.x =
                                    self.start_x + (self.end_x - self.base.position.x)
                            }

                            Overflow::Clamp => {
                                self.base.dx = 0;
                                self.base.position.x = self.end_x - 1;
                            }
                            Overflow::Oscillate => {
                                self.base.dx *= -1;
                                self.moving_forward = false;
                            }
                        }
                    }
                }
                Direction::None => panic!("platform direction cannot be none"),
            }
        } else {
            // If must be oscillating.
            let at_start = match self.direction {
                Direction::Up => self.base.position.y >= self.start_y,
                Direction::Down => self.base.position.y <= self.start_y,
                Direction::Left => self.base.position.x >= self.start_x,
                Direction::Right => self.base.position.x <= self.start_x,
                Direction::None => panic!("platform direction cannot be none"),
            };
            if at_start {
                self.moving_forward = true;
            } else {
                self.base.dx *= -1;
                self.base.dy *= -1;
            }
        }
        self.base.position.x += self.base.dx;
        self.base.position.y += self.base.dy;
    }

    fn draw(&self, batch: &mut SpriteBatch, offset: Point) {
        self.base.draw(batch, offset)
    }

    fn try_move_to(&self, player_rect: Rect, direction: Direction, is_backwards: bool) -> i32 {
        self.base.try_move_to(player_rect, direction, is_backwards)
    }
}

pub struct Bagel<'a> {
    base: PlatformBase<'a>,
    original_y: i32,
    falling: bool,
    remaining: i32,
}

impl<'a> Bagel<'a> {
    fn new<'b>(obj: MapObject, tileset: &'b TileSet<'b>) -> Result<Bagel<'b>> {
        let base = PlatformBase::new(obj, tileset)?;
        let original_y = base.position.y;
        Ok(Bagel {
            base,
            original_y,
            falling: false,
            remaining: BAGEL_WAIT_TIME,
        })
    }
}

impl<'a> Platform for Bagel<'a> {
    fn draw(&self, batch: &mut SpriteBatch, offset: Point) {
        let mut x = self.base.position.x + offset.x;
        let mut y = self.base.position.y + offset.y;
        let area = self.base.tileset.get_source_rect(self.base.tile_id);
        if self.base.occupied {
            x += (random::<u8>() % 3) as i32 - 1;
            y += (random::<u8>() % 3) as i32 - 1;
        }
        let rect = Rect {
            x,
            y,
            w: area.w * SUBPIXELS,
            h: area.h * SUBPIXELS,
        };
        batch.draw(&self.base.tileset.sprite, Some(rect), Some(area));
    }

    fn update(&mut self, switches: &mut SwitchState, sounds: &SoundManager) {
        if self.falling {
            self.remaining -= 1;
            if self.remaining == 0 {
                self.base.dy = 0;
                self.base.position.y = self.original_y;
                self.falling = false;
                self.remaining = BAGEL_WAIT_TIME;
            } else {
                self.base.dy += BAGEL_GRAVITY_ACCELERATION;
                self.base.dy = self.base.dy.max(BAGEL_MAX_GRAVITY);
                self.base.position.y += self.base.dy;
            }
        } else {
            if self.base.occupied {
                self.remaining -= 1;
                if self.remaining == 0 {
                    self.falling = true;
                    self.remaining = BAGEL_FALL_TIME;
                    self.base.dy = 0;
                }
            } else {
                self.remaining = BAGEL_WAIT_TIME;
            }
        }
    }

    fn try_move_to(&self, player_rect: Rect, direction: Direction, is_backwards: bool) -> i32 {
        self.base.try_move_to(player_rect, direction, is_backwards)
    }
}

pub struct Conveyor<'a> {
    base: PlatformBase<'a>,
}

impl<'a> Conveyor<'a> {
    fn new<'b>(obj: MapObject, tileset: &'b TileSet<'b>) -> Result<Conveyor<'b>> {
        // This is hand-tuned.
        let speed = (obj.properties.speed.unwrap_or(24) * SUBPIXELS) / 16;
        let dx = match obj.properties.convey {
            ConveyorDirection::Left => -1 * speed,
            ConveyorDirection::Right => speed,
        };
        let mut base = PlatformBase::new(obj, tileset)?;
        base.dx = dx;
        Ok(Conveyor { base: base })
    }
}

impl<'a> Platform for Conveyor<'a> {
    fn draw(&self, batch: &mut SpriteBatch, offset: Point) {}

    fn update(&mut self, switches: &mut SwitchState, sounds: &SoundManager) {}

    fn try_move_to(&self, player_rect: Rect, direction: Direction, is_backwards: bool) -> i32 {
        self.base.try_move_to(player_rect, direction, is_backwards)
    }
}

pub struct Spring<'a> {
    base: PlatformBase<'a>,
    sprite: SpriteSheet<'a>,
    up: bool,
    pos: i32,
    stall_counter: i32,
}

impl<'a> Spring<'a> {
    fn new<'b>(
        obj: MapObject,
        tileset: &'b TileSet<'b>,
        images: &'b ImageManager,
    ) -> Result<Spring<'b>> {
        let path = Path::new("assets/sprites/spring.png");
        let sprite = images.load_spritesheet(path, 8, 8)?;
        let mut base = PlatformBase::new(obj, tileset)?;
        Ok(Spring {
            base,
            sprite,
            up: false,
            pos: 0,
            stall_counter: SPRING_STALL_FRAMES,
        })
    }

    fn frame(&self) -> i32 {
        self.pos / SUBPIXELS
    }

    fn should_boost(&self) -> bool {
        self.up || (self.frame() == SPRING_STEPS - 1)
    }
}

impl<'a> Platform for Spring<'a> {
    fn draw(&self, batch: &mut SpriteBatch, offset: Point) {
        let x = self.base.position.x + offset.x;
        let y = self.base.position.y + offset.y;
        let dest = Rect {
            x,
            y,
            w: self.base.position.w,
            h: self.base.position.h,
        };
        self.sprite.blit(batch, dest, self.frame() as u32, 0, false);
    }

    fn update(&mut self, switches: &mut SwitchState, sounds: &SoundManager) {
        self.base.dx = 0;
        self.base.dy = 0;
        if !self.base.occupied {
            self.stall_counter = SPRING_STALL_FRAMES;
            self.up = false;
            if self.pos > 0 {
                self.pos -= SPRING_SPEED;
                self.base.dy = -SPRING_SPEED;
            }
        } else {
            if self.up {
                self.stall_counter = SPRING_STALL_FRAMES;
                if self.pos > 0 {
                    self.pos -= SPRING_SPEED;
                    self.base.dy = -SPRING_SPEED;
                }
            } else {
                if self.pos < (SPRING_STEPS * SUBPIXELS) - SPRING_SPEED {
                    self.stall_counter = SPRING_STALL_FRAMES;
                    self.pos += SPRING_SPEED;
                    self.base.dy = SPRING_SPEED;
                } else {
                    if self.stall_counter > 0 {
                        self.stall_counter -= 1;
                    } else {
                        self.stall_counter = SPRING_STALL_FRAMES;
                        self.up = true;
                    }
                }
            }
        }
    }

    fn try_move_to(&self, player_rect: Rect, direction: Direction, is_backwards: bool) -> i32 {
        if self.base.solid {
            let area = Rect {
                x: self.base.position.x,
                y: self.base.position.y + self.pos,
                w: self.base.position.w,
                h: self.base.position.h - self.pos,
            };
            try_move_to_bounds(player_rect, area, direction)
        } else {
            if !matches!(direction, Direction::Down) {
                return 0;
            }
            if is_backwards {
                return 0;
            }
            let area = Rect {
                x: self.base.position.x,
                y: self.base.position.y + self.pos,
                w: self.base.position.w,
                h: self.base.position.h / 2,
            };
            try_move_to_bounds(player_rect, area, direction)
        }
    }
}

struct Button<'a> {
    base: PlatformBase<'a>,
    sprite: SpriteSheet<'a>,
    level: u32,
    original_y: i32,
    clicked: bool,
    button_type: ButtonType,
    was_occupied: bool,
    color: String,
}

fn get_button_image_path(color: &str) -> String {
    let color = if color == "!white" { "black" } else { "white" };
    format!("assets/sprites/buttons/{color}.png")
}

impl<'a> Button<'a> {
    fn new<'b>(
        obj: MapObject,
        tileset: &'b TileSet<'b>,
        images: &'b ImageManager<'b>,
    ) -> Result<Button<'b>> {
        let level = 0;
        let clicked = false;
        let was_occupied = false;

        let color = obj
            .properties
            .color
            .clone()
            .unwrap_or_else(|| "red".to_string());
        let image_path = get_button_image_path(&color);
        let sprite = images.load_spritesheet(Path::new(&image_path), 8, 8)?;
        let button_type = obj.properties.button_type;

        let mut base = PlatformBase::new(obj, tileset)?;
        let original_y = base.position.y;
        // Move down by a whole pixel while on a button.
        base.dy = SUBPIXELS;

        Ok(Button {
            base,
            sprite,
            level,
            original_y,
            clicked,
            button_type,
            was_occupied,
            color,
        })
    }
}

impl<'a> Platform for Button<'a> {
    fn draw(&self, batch: &mut SpriteBatch, offset: Point) {
        let x = self.base.position.x + offset.x;
        let y = self.original_y + offset.y();
        let dest = Rect {
            x,
            y,
            w: self.base.position.w,
            h: self.base.position.h,
        };
        self.sprite
            .blit(batch, dest, self.level / BUTTON_DELAY, 0, false);
    }

    fn update(&mut self, switches: &mut SwitchState, sounds: &SoundManager) {
        let was_clicked = self.clicked;

        if matches!(self.button_type, ButtonType::Smart) {
            self.clicked = switches.is_condition_true(&self.color);
        }

        if self.base.occupied && !self.was_occupied {
            self.clicked = match self.button_type {
                ButtonType::OneShot | ButtonType::Smart => true,
                ButtonType::Toggle => !self.clicked,
                _ => self.clicked,
            };
        }

        self.was_occupied = self.base.occupied;

        if matches!(self.button_type, ButtonType::Momentary) {
            self.clicked = self.base.occupied;
        }

        if self.clicked {
            if self.level < BUTTON_MAX_LEVEL {
                self.level += 1;
            }
        } else {
            if self.level > 0 {
                self.level -= 1;
            }
        }

        self.base.position.y =
            self.original_y + ((self.level * SUBPIXELS as u32) / BUTTON_DELAY) as i32;

        if self.clicked != was_clicked {
            // sounds.play(Sound.CLICK);
            if matches!(self.button_type, ButtonType::Smart) {
                if self.clicked && self.base.occupied {
                    switches.apply_command(&self.color);
                }
            } else if self.clicked || !matches!(self.button_type, ButtonType::OneShot) {
                switches.toggle(&self.color);
            }
        }
    }

    fn try_move_to(&self, player_rect: Rect, direction: Direction, is_backwards: bool) -> i32 {
        self.base.try_move_to(player_rect, direction, is_backwards)
    }
}
