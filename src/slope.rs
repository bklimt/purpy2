use crate::tileset::TileProperties;
use crate::utils::{Direction, Rect, Subpixels};

use anyhow::Result;

pub struct Slope {
    left_y: Subpixels,
    right_y: Subpixels,
}

impl Slope {
    pub fn new(properties: &TileProperties) -> Result<Self> {
        let left_y = properties.left_y;
        let right_y = properties.right_y;
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
                println!("");
                println!("direction = {:?}", direction);
                println!("center_x = {actor_center_x}");
                println!("x_offset = {}", x_offset / 16);
                println!("slope = {slope}");
                println!("target_y = {}", target_y / 16);
                println!("actor_bottom = {}", actor.bottom() / 16);
                if target_y < actor.bottom() {
                    println!("pushing actor by {}", target_y - actor.bottom());
                }
            }
        }

        if target_y < actor.bottom() {
            target_y - actor.bottom()
        } else {
            0
        }
    }
}
