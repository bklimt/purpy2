use crate::geometry::{Rect, Subpixels};
use crate::tileset::TileProperties;
use crate::utils::Direction;

use anyhow::Result;
use num_traits::Zero;

pub struct Slope {
    pub left_y: Subpixels,
    pub right_y: Subpixels,
}

impl Slope {
    pub fn new(properties: &TileProperties) -> Result<Self> {
        let left_y = properties.left_y.into();
        let right_y = properties.right_y.into();
        Ok(Slope { left_y, right_y })
    }

    /*
     * Try to move the actor rect in direction by delta and see if it intersects target.
     *
     * Returns the maximum distance the actor can move.
     */
    pub fn try_move_to_bounds(
        &self,
        actor: Rect<Subpixels>,
        target: Rect<Subpixels>,
        direction: Direction,
    ) -> Subpixels {
        let left_y = self.left_y;
        let right_y = self.right_y;

        if actor.bottom() <= target.top() {
            return Subpixels::zero();
        } else if actor.top() >= target.bottom() {
            return Subpixels::zero();
        } else if actor.right() <= target.left() {
            return Subpixels::zero();
        } else if actor.left() >= target.right() {
            return Subpixels::zero();
        }

        let Direction::Down = direction else {
            return Subpixels::zero();
        };

        let actor_center_x = (actor.left() + actor.right()) / 2;

        let target_y = if actor_center_x < target.left() {
            target.top() + left_y
        } else if actor_center_x > target.right() {
            target.top() + right_y
        } else {
            let x_offset = actor_center_x - target.x;
            // A hacky way to divide two subpixels and get a float.
            let one_subpixel = Subpixels::new(1);
            let d_y_f = ((right_y - left_y) / one_subpixel) as f32;
            let target_w_f = (target.w / one_subpixel) as f32;
            let slope = d_y_f / target_w_f;
            if false {
                println!("");
                println!("direction = {:?}", direction);
                println!("center_x = {:?}", actor_center_x);
                println!("x_offset = {:?}", x_offset / 16);
                println!("slope = {:?}", slope);
                println!("actor_bottom = {:?}", actor.bottom() / 16);
            }

            // A hacky way to multiply a float times subpixels without losing precision.
            let x_offset_f = (x_offset / Subpixels::new(1)) as f32;
            let adjusted_x_offset = slope * x_offset_f;
            let adjusted_x_offset = Subpixels::new(adjusted_x_offset as i32);

            target.y + adjusted_x_offset + left_y
        };

        if target_y < actor.bottom() {
            target_y - actor.bottom()
        } else {
            Subpixels::zero()
        }
    }
}
