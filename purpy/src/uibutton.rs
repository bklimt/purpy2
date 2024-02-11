use std::path::Path;

use anyhow::Result;
use log::info;

use crate::font::Font;
use crate::geometry::Pixels;
use crate::geometry::Point;
use crate::geometry::Rect;
use crate::geometry::Subpixels;
use crate::imagemanager::ImageLoader;
use crate::inputmanager::InputSnapshot;
use crate::rendercontext::RenderContext;
use crate::rendercontext::RenderLayer;
use crate::soundmanager::Sound;
use crate::soundmanager::SoundManager;
use crate::sprite::SpriteSheet;
use crate::tilemap::MapObject;

#[derive(Debug, Clone, Copy)]
enum UiButtonState {
    Normal = 0,
    Hover = 1,
    MouseClick = 2,
    GamepadClick = 3,
}

pub struct UiButton {
    position: Rect<Subpixels>,
    sprite: SpriteSheet,
    state: UiButtonState,
    label: String,
    action: Option<String>,
    tile_width: Pixels,
    tile_height: Pixels,
}

impl UiButton {
    pub fn new(
        object: &MapObject,
        tile_width: Pixels,
        tile_height: Pixels,
        images: &mut dyn ImageLoader,
    ) -> Result<Self> {
        let position = object.position.as_subpixels();
        let sprite =
            images.load_spritesheet(Path::new("assets/uibutton.png"), tile_width, tile_height)?;
        let state = UiButtonState::Normal;
        let label = object.properties.label.clone();
        let action = object.properties.uibutton.clone();
        Ok(UiButton {
            position,
            sprite,
            state,
            label,
            action,
            tile_width,
            tile_height,
        })
    }

    pub fn update(
        &mut self,
        selected: bool,
        inputs: &InputSnapshot,
        sounds: &mut SoundManager,
    ) -> Option<String> {
        let mut clicked = false;
        let mouse_inside = self.position.contains(inputs.mouse_position.into());

        self.state = if matches!(self.state, UiButtonState::MouseClick) {
            if inputs.mouse_button_left_down {
                self.state
            } else {
                if mouse_inside {
                    info!("uibutton clicked");
                    clicked = true;
                }
                UiButtonState::Normal
            }
        } else if matches!(self.state, UiButtonState::GamepadClick) {
            if inputs.ok_down {
                self.state
            } else {
                if mouse_inside {
                    info!("uibutton clicked");
                    clicked = true;
                }
                UiButtonState::Normal
            }
        } else if selected && inputs.ok_down {
            UiButtonState::GamepadClick
        } else if mouse_inside && inputs.mouse_button_left_down {
            UiButtonState::MouseClick
        } else if selected || mouse_inside {
            UiButtonState::Hover
        } else {
            UiButtonState::Normal
        };

        if clicked {
            sounds.play(Sound::Click);
            self.action.clone()
        } else {
            None
        }
    }

    pub fn draw(&self, context: &mut RenderContext, layer: RenderLayer, font: &Font) {
        let w = self.tile_width.as_subpixels();
        let h = self.tile_height.as_subpixels();
        let cols = self.position.w / w;
        let rows = self.position.h / h;
        for row in 0..rows {
            for col in 0..cols {
                let x = self.position.x + w * col;
                let y = self.position.y + h * row;
                let dest = Rect { x, y, w, h };

                // Do 9-slice logic.
                let index = if col == 0 {
                    0
                } else if col == cols - 1 {
                    2
                } else {
                    1
                };
                let sprite_layer = if row == 0 {
                    0
                } else if row == rows - 1 {
                    2
                } else {
                    1
                };

                // Apply the state.
                let index = index
                    + match self.state {
                        UiButtonState::Normal => 0,
                        UiButtonState::Hover => 3,
                        UiButtonState::GamepadClick | UiButtonState::MouseClick => 6,
                    };

                self.sprite
                    .blit(context, layer, dest, index, sprite_layer, false);
            }
        }

        let mut label_pos = self.position.top_left();
        label_pos += Point::new(font.char_width, font.char_height);
        if matches!(
            self.state,
            UiButtonState::MouseClick | UiButtonState::GamepadClick
        ) {
            label_pos += Point::new(Subpixels::from_pixels(2), Subpixels::from_pixels(2));
        }
        font.draw_string(context, layer, label_pos, &self.label);
    }
}
