use std::str::FromStr;

use anyhow::{anyhow, bail, Error};

pub type Subpixels = i32;

#[derive(Clone, Copy)]
pub struct Point {
    pub x: Subpixels,
    pub y: Subpixels,
}

impl Point {
    pub fn new(x: Subpixels, y: Subpixels) -> Point {
        Point { x, y }
    }

    pub fn x(&self) -> Subpixels {
        self.x
    }

    pub fn y(&self) -> Subpixels {
        self.y
    }
}

impl From<(i32, i32)> for Point {
    fn from(value: (i32, i32)) -> Self {
        Point::new(value.0, value.1)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    None,
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn opposite(&self) -> Direction {
        match self {
            Direction::None => panic!("cannot take the opposite of no direction"),
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Right => Direction::Left,
            Direction::Left => Direction::Right,
        }
    }
}

impl FromStr for Direction {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "N" => Direction::Up,
            "S" => Direction::Down,
            "W" => Direction::Left,
            "E" => Direction::Right,
            _ => bail!("invalid direction: {}", s),
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl FromStr for Color {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = if s.starts_with("#") { &s[1..] } else { s };
        if s.len() == 6 {
            let r = u8::from_str_radix(&s[0..2], 16)?;
            let g = u8::from_str_radix(&s[2..4], 16)?;
            let b = u8::from_str_radix(&s[4..6], 16)?;
            Ok(Color { r, g, b, a: 255 })
        } else if s.len() == 8 {
            let r = u8::from_str_radix(&s[0..2], 16)?;
            let g = u8::from_str_radix(&s[2..4], 16)?;
            let b = u8::from_str_radix(&s[4..6], 16)?;
            let a = u8::from_str_radix(&s[6..8], 16)?;
            Ok(Color { r, g, b, a })
        } else {
            Err(anyhow!("invalid color: {}", s))
        }
    }
}

impl From<Color> for sdl2::pixels::Color {
    fn from(value: Color) -> Self {
        sdl2::pixels::Color::RGBA(value.r, value.g, value.b, value.a)
    }
}

pub fn sign(n: Subpixels) -> Subpixels {
    if n < 0 {
        -1
    } else if n > 0 {
        1
    } else {
        0
    }
}

pub fn cmp_in_direction(a: Subpixels, b: Subpixels, direction: Direction) -> Subpixels {
    match direction {
        Direction::Up | Direction::Left => sign(b - a),
        _ => sign(a - b),
    }
}

#[derive(Clone, Copy)]
pub struct Rect {
    pub x: Subpixels,
    pub y: Subpixels,
    pub w: Subpixels,
    pub h: Subpixels,
}

impl Rect {
    pub fn top(&self) -> Subpixels {
        self.y
    }
    pub fn bottom(&self) -> Subpixels {
        self.y + self.h
    }
    pub fn left(&self) -> Subpixels {
        self.x
    }
    pub fn right(&self) -> Subpixels {
        self.x + self.w
    }
}

impl Into<sdl2::rect::Rect> for Rect {
    fn into(self) -> sdl2::rect::Rect {
        sdl2::rect::Rect::new(self.x, self.y, self.w as u32, self.h as u32)
    }
}

impl Into<Option<sdl2::rect::Rect>> for Rect {
    fn into(self) -> Option<sdl2::rect::Rect> {
        Some(sdl2::rect::Rect::new(
            self.x,
            self.y,
            self.w as u32,
            self.h as u32,
        ))
    }
}

/*
 * Try to move the actor rect in direction by delta and see if it intersects target.
 *
 * Returns the maximum distance the actor can move.
 */
pub fn try_move_to_bounds(actor: Rect, target: Rect, direction: Direction) -> Subpixels {
    if actor.bottom() <= target.top() {
        0
    } else if actor.top() >= target.bottom() {
        0
    } else if actor.right() <= target.left() {
        0
    } else if actor.left() >= target.right() {
        0
    } else {
        match direction {
            Direction::None => panic!("cannot try_move_to in no direction"),
            Direction::Up => target.bottom() - actor.top(),
            Direction::Down => target.top() - actor.bottom(),
            Direction::Right => target.left() - actor.right(),
            Direction::Left => target.right() - actor.left(),
        }
    }
}

pub fn intersect(rect1: Rect, rect2: Rect) -> bool {
    if rect1.right() < rect2.left() {
        false
    } else if rect1.left() > rect2.right() {
        false
    } else if rect1.bottom() < rect2.top() {
        false
    } else if rect1.top() > rect2.bottom() {
        false
    } else {
        true
    }
}
