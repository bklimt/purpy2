use std::mem;
use std::path::Path;
use std::rc::Rc;

use anyhow::{Context, Result};
use rand::random;

use crate::constants::{
    BAGEL_FALL_TIME, BAGEL_GRAVITY_ACCELERATION, BAGEL_MAX_GRAVITY, BAGEL_WAIT_TIME, BUTTON_DELAY,
    BUTTON_MAX_LEVEL, SPRING_SPEED, SPRING_STALL_FRAMES, SPRING_STEPS, SUBPIXELS,
};
use crate::imagemanager::ImageManager;
use crate::rendercontext::{RenderContext, RenderLayer};
use crate::soundmanager::{Sound, SoundManager};
use crate::sprite::SpriteSheet;
use crate::switchstate::SwitchState;
use crate::tilemap::{ButtonType, ConveyorDirection, MapObject, Overflow};
use crate::tileset::{TileIndex, TileSet};
use crate::utils::{sign, try_move_to_bounds, Direction, Point, Rect};

pub enum PlatformType<'a> {
    MovingPlatform(MovingPlatform),
    Bagel(Bagel),
    Conveyor(Conveyor),
    Spring(Spring<'a>),
    Button(Button<'a>),
}

pub struct Platform<'a> {
    _id: i32,
    tileset: Rc<TileSet<'a>>,
    tile_id: TileIndex,
    position: Rect,
    dx: i32,
    dy: i32,
    solid: bool,
    occupied: bool,
    pub subtype: PlatformType<'a>,
}

impl<'a> Platform<'a> {
    fn new<'b>(
        obj: &MapObject,
        tileset: Rc<TileSet<'b>>,
        subtype: PlatformType<'b>,
    ) -> Result<Platform<'b>> {
        Ok(Platform {
            _id: obj.id,
            tileset: tileset,
            tile_id: obj.gid.context("gid required for platforms")? as TileIndex - 1,
            position: Rect {
                x: obj.position.x * SUBPIXELS,
                y: obj.position.y * SUBPIXELS,
                w: obj.position.w * SUBPIXELS,
                h: obj.position.h * SUBPIXELS,
            },
            dx: 0,
            dy: 0,
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
            _ => unimplemented!(),
        }
        self.subtype = subtype;
    }

    pub fn draw<'b>(&self, context: &'b mut RenderContext<'a>, layer: RenderLayer, offset: Point)
    where
        'a: 'b,
    {
        match &self.subtype {
            PlatformType::Bagel(bagel) => bagel.draw(self, context, layer, offset),
            PlatformType::Spring(spring) => spring.draw(self, context, layer, offset),
            PlatformType::Button(button) => button.draw(self, context, layer, offset),
            _ => {
                let x = self.position.x + offset.x();
                let y = self.position.y + offset.y();
                let dest = Rect {
                    x,
                    y,
                    w: self.position.w,
                    h: self.position.h,
                };
                if let Some(anim) = self.tileset.animations.get(&self.tile_id) {
                    anim.blit(context, layer, dest, false);
                } else {
                    let src = self.tileset.get_source_rect(self.tile_id);
                    context.draw(&self.tileset.sprite, layer, dest, src);
                }
            }
        }
    }

    pub fn try_move_to(&self, player_rect: Rect, direction: Direction, is_backwards: bool) -> i32 {
        if let PlatformType::Spring(spring) = &self.subtype {
            return spring.try_move_to(self, player_rect, direction, is_backwards);
        }

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

    pub fn is_solid(&self) -> bool {
        self.solid
    }
    pub fn dx(&self) -> i32 {
        self.dx
    }
    pub fn dy(&self) -> i32 {
        self.dy
    }
    pub fn set_occupied(&mut self, occupied: bool) {
        self.occupied = occupied;
    }
}

pub struct MovingPlatform {
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

impl MovingPlatform {
    pub fn new<'b>(obj: &MapObject, tileset: Rc<TileSet<'b>>) -> Result<Platform<'b>> {
        let (dist_mult, sx, sy) = match obj.properties.direction {
            Direction::Up => (tileset.tileheight, 0, -1),
            Direction::Down => (tileset.tileheight, 0, 1),
            Direction::Left => (tileset.tilewidth, -1, 0),
            Direction::Right => (tileset.tilewidth, 1, 0),
            Direction::None => (0, 0, 0),
        };

        // This is 16 for historical reasons, just because that's what the speed is tuned for.
        let speed = obj.properties.speed.unwrap_or(1);
        let speed = (speed * SUBPIXELS) / 16;
        let dist_mult = dist_mult * SUBPIXELS;
        let distance = obj.properties.distance * dist_mult;
        let direction = obj.properties.direction;
        let start_x = obj.position.x * SUBPIXELS;
        let start_y = obj.position.y * SUBPIXELS;
        let end_x = start_x + sx * distance;
        let end_y = start_y + sy * distance;
        let moving_forward = true;
        let condition = obj.properties.condition.clone();
        let overflow = obj.properties.overflow;

        let moving_platform = MovingPlatform {
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
        };
        Ok(Platform::new(
            obj,
            tileset,
            PlatformType::MovingPlatform(moving_platform),
        )?)
    }

    fn update(&mut self, base: &mut Platform, switches: &mut SwitchState, sounds: &SoundManager) {
        if let Some(condition) = self.condition.as_deref() {
            if !switches.is_condition_true(&condition) {
                self.moving_forward = false;
                if base.position.x == self.start_x && base.position.y == self.start_y {
                    base.dx = 0;
                    base.dy = 0;
                    return;
                }
            }
        }

        base.dx = sign(self.end_x - self.start_x) * self.speed;
        base.dy = sign(self.end_y - self.start_y) * self.speed;
        if self.moving_forward {
            match self.direction {
                Direction::Up => {
                    if base.position.y <= self.end_y {
                        match self.overflow {
                            Overflow::Wrap => base.position.y += self.distance,
                            Overflow::Clamp => {
                                base.dy = 0;
                                base.position.y = self.end_y + 1;
                            }
                            Overflow::Oscillate => {
                                base.dy *= -1;
                                self.moving_forward = false;
                            }
                        }
                    }
                }
                Direction::Down => {
                    if base.position.y >= self.end_y {
                        match self.overflow {
                            Overflow::Wrap => {
                                base.position.y = self.start_y + (self.end_y - base.position.y)
                            }

                            Overflow::Clamp => {
                                base.dy = 0;
                                base.position.y = self.end_y - 1;
                            }
                            Overflow::Oscillate => {
                                base.dy *= -1;
                                self.moving_forward = false;
                            }
                        }
                    }
                }
                Direction::Left => {
                    if base.position.x <= self.end_x {
                        match self.overflow {
                            Overflow::Wrap => base.position.x += self.distance,

                            Overflow::Clamp => {
                                base.dx = 0;
                                base.position.x = self.end_x + 1;
                            }
                            Overflow::Oscillate => {
                                base.dx *= -1;
                                self.moving_forward = false;
                            }
                        }
                    }
                }
                Direction::Right => {
                    if base.position.x >= self.end_x {
                        match self.overflow {
                            Overflow::Wrap => {
                                base.position.x = self.start_x + (self.end_x - base.position.x)
                            }

                            Overflow::Clamp => {
                                base.dx = 0;
                                base.position.x = self.end_x - 1;
                            }
                            Overflow::Oscillate => {
                                base.dx *= -1;
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
                Direction::Up => base.position.y >= self.start_y,
                Direction::Down => base.position.y <= self.start_y,
                Direction::Left => base.position.x >= self.start_x,
                Direction::Right => base.position.x <= self.start_x,
                Direction::None => panic!("platform direction cannot be none"),
            };
            if at_start {
                self.moving_forward = true;
            } else {
                base.dx *= -1;
                base.dy *= -1;
            }
        }
        base.position.x += base.dx;
        base.position.y += base.dy;
    }
}

pub struct Bagel {
    original_y: i32,
    falling: bool,
    remaining: i32,
}

impl Bagel {
    pub fn new<'b>(obj: &MapObject, tileset: Rc<TileSet<'b>>) -> Result<Platform<'b>> {
        let original_y = obj.position.y * SUBPIXELS;
        let bagel = Bagel {
            original_y,
            falling: false,
            remaining: BAGEL_WAIT_TIME,
        };
        Ok(Platform::new(obj, tileset, PlatformType::Bagel(bagel))?)
    }

    fn draw<'a, 'b>(
        &self,
        base: &Platform<'a>,
        context: &'b mut RenderContext<'a>,
        layer: RenderLayer,
        offset: Point,
    ) where
        'a: 'b,
    {
        let mut x = base.position.x + offset.x;
        let mut y = base.position.y + offset.y;
        let area = base.tileset.get_source_rect(base.tile_id);
        if base.occupied {
            x += (random::<u8>() % 3) as i32 - 1;
            y += (random::<u8>() % 3) as i32 - 1;
        }
        let rect = Rect {
            x,
            y,
            w: area.w * SUBPIXELS,
            h: area.h * SUBPIXELS,
        };
        context.draw(&base.tileset.sprite, layer, rect, area);
    }

    fn update(&mut self, base: &mut Platform, switches: &mut SwitchState, sounds: &SoundManager) {
        if self.falling {
            self.remaining -= 1;
            if self.remaining == 0 {
                base.dy = 0;
                base.position.y = self.original_y;
                self.falling = false;
                self.remaining = BAGEL_WAIT_TIME;
            } else {
                base.dy += BAGEL_GRAVITY_ACCELERATION;
                base.dy = base.dy.max(BAGEL_MAX_GRAVITY);
                base.position.y += base.dy;
            }
        } else {
            if base.occupied {
                self.remaining -= 1;
                if self.remaining == 0 {
                    self.falling = true;
                    self.remaining = BAGEL_FALL_TIME;
                    base.dy = 0;
                }
            } else {
                self.remaining = BAGEL_WAIT_TIME;
            }
        }
    }
}

pub struct Conveyor(());

impl Conveyor {
    pub fn new<'b>(obj: &MapObject, tileset: Rc<TileSet<'b>>) -> Result<Platform<'b>> {
        // This is hand-tuned.
        let speed = (obj.properties.speed.unwrap_or(24) * SUBPIXELS) / 16;
        let dx = match obj
            .properties
            .convey
            .expect("conveyor does not have convey property")
        {
            ConveyorDirection::Left => -1 * speed,
            ConveyorDirection::Right => speed,
        };

        let mut base = Platform::new(obj, tileset, PlatformType::Conveyor(Conveyor(())))?;
        base.dx = dx;
        Ok(base)
    }
}

pub struct Spring<'a> {
    sprite: SpriteSheet<'a>,
    up: bool,
    pos: i32,
    stall_counter: i32,
    pub launch: bool,
}

impl<'a> Spring<'a> {
    pub fn new<'b, 'c>(
        obj: &MapObject,
        tileset: Rc<TileSet<'b>>,
        images: &'c ImageManager<'b>,
    ) -> Result<Platform<'b>>
    where
        'b: 'c,
    {
        let path = Path::new("assets/sprites/spring.png");
        let sprite = images.load_spritesheet(path, 8, 8)?;
        let spring = Spring {
            sprite,
            up: false,
            pos: 0,
            stall_counter: SPRING_STALL_FRAMES,
            launch: false,
        };
        Ok(Platform::new(obj, tileset, PlatformType::Spring(spring))?)
    }

    fn frame(&self) -> i32 {
        self.pos / SUBPIXELS
    }

    pub fn should_boost(&self) -> bool {
        self.up || (self.frame() == SPRING_STEPS - 1)
    }

    fn draw<'b>(
        &self,
        base: &Platform<'a>,
        context: &'b mut RenderContext<'a>,
        layer: RenderLayer,
        offset: Point,
    ) where
        'a: 'b,
    {
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

    fn update(&mut self, base: &mut Platform, switches: &mut SwitchState, sounds: &SoundManager) {
        base.dx = 0;
        base.dy = 0;
        self.launch = false;
        if !base.occupied {
            self.stall_counter = SPRING_STALL_FRAMES;
            self.up = false;
            if self.pos > 0 {
                self.pos -= SPRING_SPEED;
                base.dy = -SPRING_SPEED;
            }
        } else {
            if self.up {
                self.stall_counter = SPRING_STALL_FRAMES;
                if self.pos > 0 {
                    self.pos -= SPRING_SPEED;
                    base.dy = -SPRING_SPEED;
                } else {
                    self.launch = true;
                }
            } else {
                if self.pos < (SPRING_STEPS * SUBPIXELS) - SPRING_SPEED {
                    self.stall_counter = SPRING_STALL_FRAMES;
                    self.pos += SPRING_SPEED;
                    base.dy = SPRING_SPEED;
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

    fn try_move_to(
        &self,
        base: &Platform,
        player_rect: Rect,
        direction: Direction,
        is_backwards: bool,
    ) -> i32 {
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
                return 0;
            }
            if is_backwards {
                return 0;
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

pub struct Button<'a> {
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
    pub fn new<'b, 'c>(
        obj: &MapObject,
        tileset: Rc<TileSet<'b>>,
        images: &'c ImageManager<'b>,
    ) -> Result<Platform<'b>>
    where
        'b: 'c,
    {
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

        let original_y = obj.position.y * SUBPIXELS;
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
        base.dy = SUBPIXELS;
        Ok(base)
    }

    fn draw<'b>(
        &self,
        base: &Platform<'a>,
        context: &'b mut RenderContext<'a>,
        layer: RenderLayer,
        offset: Point,
    ) where
        'a: 'b,
    {
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
        } else {
            if self.level > 0 {
                self.level -= 1;
            }
        }

        base.position.y = self.original_y + ((self.level * SUBPIXELS as u32) / BUTTON_DELAY) as i32;

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
