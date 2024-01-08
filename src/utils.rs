use std::str::FromStr;

use anyhow::{anyhow, Error};

pub type Subpixels = i32;

#[derive(Clone, Copy)]
pub struct Point(Subpixels, Subpixels);

impl Point {
    pub fn new(x: Subpixels, y: Subpixels) -> Point {
        Point(x, y)
    }

    pub fn x(&self) -> Subpixels {
        self.0
    }

    pub fn y(&self) -> Subpixels {
        self.1
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
    fn opposite(&self) -> Direction {
        match self {
            Direction::None => panic!("cannot take the opposite of no direction"),
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Right => Direction::Left,
            Direction::Left => Direction::Right,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl FromStr for Color {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = if s.starts_with("#") { &s[1..] } else { s };
        if s.len() == 6 {
            let r = s[0..2].parse()?;
            let g = s[2..4].parse()?;
            let b = s[4..6].parse()?;
            Ok(Color { r, g, b, a: 255 })
        } else if s.len() == 8 {
            let r = s[0..2].parse()?;
            let g = s[2..4].parse()?;
            let b = s[4..6].parse()?;
            let a = s[6..8].parse()?;
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

fn sign(n: Subpixels) -> Subpixels {
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

/*
 * Try to move the actor rect in direction by delta and see if it intersects target.
 *
 * Returns the maximum distance the actor can move.
 */
pub fn try_move_to_slope_bounds(
    actor: &Rect,
    target: &Rect,
    left_y: Subpixels,
    right_y: Subpixels,
    direction: &Direction,
) -> Subpixels {
    if actor.bottom() <= target.top() {
        return 0;
    }
    if actor.top() >= target.bottom() {
        return 0;
    }
    if actor.right() <= target.left() {
        return 0;
    }
    if actor.left() >= target.right() {
        return 0;
    }

    if let Direction::Down = direction {
        return 0;
    }

    let mut target_y = actor.bottom();
    let actor_center_x = (actor.left() + actor.right()) / 2;

    if actor_center_x < target.left() {
        target_y = target.top() + left_y;
    } else if actor_center_x > target.right() {
        target_y = target.top() + right_y;
    } else {
        let x_offset = actor_center_x - target.x;
        let slope = (right_y - left_y) / target.w;
        target_y = target.y + slope * x_offset + left_y;

        if false {
            println!("center_x = {actor_center_x}");
            println!("x_offset = {x_offset}");
            println!("slope = {slope}");
            println!("target_y = {target_y}");
            println!("actor_bottom = {}", actor.bottom());
        }
    }

    if target_y < actor.bottom() {
        target_y - actor.bottom()
    } else {
        0
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

fn inside(rect: Rect, point: Point) -> bool {
    if point.0 < rect.left() || point.0 > rect.right() {
        false
    } else if point.1 < rect.top() || point.1 > rect.bottom() {
        false
    } else {
        true
    }
}
