use crate::constants::SUBPIXELS;
use crate::tileset::TileProperties;
use crate::utils::{Direction, Rect, Subpixels};

use anyhow::Result;

pub struct Slope {
    pub left_y: Subpixels,
    pub right_y: Subpixels,
}

impl Slope {
    pub fn new(properties: &TileProperties) -> Result<Self> {
        let left_y = properties.left_y * SUBPIXELS;
        let right_y = properties.right_y * SUBPIXELS;
        Ok(Slope { left_y, right_y })
    }

    /*
     * Try to move the actor rect in direction by delta and see if it intersects target.
     *
     * Returns the maximum distance the actor can move.
     */
    pub fn try_move_to_bounds(&self, actor: Rect, target: Rect, direction: Direction) -> Subpixels {
        let left_y = self.left_y;
        let right_y = self.right_y;

        if actor.bottom() <= target.top() {
            return 0;
        } else if actor.top() >= target.bottom() {
            return 0;
        } else if actor.right() <= target.left() {
            return 0;
        } else if actor.left() >= target.right() {
            return 0;
        }

        let Direction::Down = direction else {
            return 0;
        };

        let actor_center_x = (actor.left() + actor.right()) / 2;

        let target_y = if actor_center_x < target.left() {
            target.top() + left_y
        } else if actor_center_x > target.right() {
            target.top() + right_y
        } else {
            let x_offset = actor_center_x - target.x;
            let slope = (right_y - left_y) as f32 / target.w as f32;
            if false {
                println!("");
                println!("direction = {:?}", direction);
                println!("center_x = {actor_center_x}");
                println!("x_offset = {}", x_offset / 16);
                println!("slope = {slope}");
                println!("actor_bottom = {}", actor.bottom() / 16);
            }
            target.y + (slope * x_offset as f32) as i32 + left_y
        };

        if target_y < actor.bottom() {
            target_y - actor.bottom()
        } else {
            0
        }
    }
}
