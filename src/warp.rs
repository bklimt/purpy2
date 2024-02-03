use anyhow::{Context, Result};

use crate::{
    geometry::{Rect, Subpixels},
    tilemap::MapObject,
};

pub struct Warp {
    position: Rect<Subpixels>,
    pub destination: String,
}

impl Warp {
    pub fn new(obj: &MapObject) -> Result<Self> {
        Ok(Self {
            position: obj.position.into(),
            destination: obj
                .properties
                .warp
                .as_ref()
                .context("destination required for warp")?
                .clone(),
        })
    }

    pub fn is_inside(&self, player_rect: Rect<Subpixels>) -> bool {
        player_rect.intersects(self.position)
    }
}
