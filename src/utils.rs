use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{anyhow, bail, Error, Result};
use num_traits::Zero;

use crate::geometry::{Rect, Subpixels};

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn opposite(&self) -> Direction {
        match self {
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

impl From<Color> for wgpu::Color {
    fn from(value: Color) -> Self {
        wgpu::Color {
            r: value.r as f64 / 255.0,
            g: value.g as f64 / 255.0,
            b: value.b as f64 / 255.0,
            a: value.a as f64 / 255.0,
        }
    }
}

impl From<Color> for [f32; 4] {
    fn from(value: Color) -> Self {
        [
            value.r as f32 / 255.0,
            value.g as f32 / 255.0,
            value.b as f32 / 255.0,
            value.a as f32 / 255.0,
        ]
    }
}

pub fn cmp_in_direction(a: Subpixels, b: Subpixels, direction: Direction) -> Ordering {
    let sign = match direction {
        Direction::Up | Direction::Left => b - a,
        _ => a - b,
    };
    if sign < Subpixels::zero() {
        Ordering::Less
    } else if sign > Subpixels::zero() {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

/*
 * Try to move the actor rect in direction by delta and see if it intersects target.
 *
 * Returns the maximum distance the actor can move.
 */
pub fn try_move_to_bounds(
    actor: Rect<Subpixels>,
    target: Rect<Subpixels>,
    direction: Direction,
) -> Subpixels {
    if actor.bottom() <= target.top() {
        Subpixels::zero()
    } else if actor.top() >= target.bottom() {
        Subpixels::zero()
    } else if actor.right() <= target.left() {
        Subpixels::zero()
    } else if actor.left() >= target.right() {
        Subpixels::zero()
    } else {
        match direction {
            Direction::Up => target.bottom() - actor.top(),
            Direction::Down => target.top() - actor.bottom(),
            Direction::Right => target.left() - actor.right(),
            Direction::Left => target.right() - actor.left(),
        }
    }
}

pub fn intersect<T>(rect1: Rect<T>, rect2: Rect<T>) -> bool
where
    T: std::ops::Add<T, Output = T> + PartialOrd + Copy,
{
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

pub fn normalize_path(path: &Path) -> Result<PathBuf> {
    let mut output = PathBuf::new();
    for part in path.into_iter() {
        if part == ".." {
            if !output.pop() {
                output.push(part);
            }
        } else {
            output.push(part);
        }
    }
    Ok(output)
}
