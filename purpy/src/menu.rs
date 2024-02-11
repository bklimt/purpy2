use std::path::Path;

use anyhow::Result;
use log::error;
use num_traits::Zero;

use crate::cursor::Cursor;
use crate::filemanager::FileManager;
use crate::font::Font;
use crate::geometry::Point;
use crate::imagemanager::ImageLoader;
use crate::inputmanager::InputSnapshot;
use crate::rendercontext::{RenderContext, RenderLayer};
use crate::scene::{Scene, SceneResult};
use crate::soundmanager::SoundManager;
use crate::switchstate::SwitchState;
use crate::tilemap::TileMap;
use crate::uibutton::UiButton;
use crate::utils::Color;

pub struct Menu {
    cursor: Cursor,
    tilemap: TileMap,
    buttons: Vec<UiButton>,
    selected: usize,
    switches: SwitchState,
}

impl Menu {
    pub fn new(path: &Path, files: &FileManager, images: &mut dyn ImageLoader) -> Result<Self> {
        let cursor = Cursor::new(images)?;
        let tilemap = TileMap::from_file(path, files, images)?;
        let mut buttons = Vec::new();
        let selected = 0;
        let switches = SwitchState::new();

        for obj in &tilemap.objects {
            if let Some(_) = &obj.properties.uibutton {
                buttons.push(UiButton::new(
                    obj,
                    tilemap.tilewidth,
                    tilemap.tileheight,
                    images,
                )?);
            }
        }

        Ok(Self {
            cursor,
            tilemap,
            buttons,
            selected,
            switches,
        })
    }
}

impl Scene for Menu {
    fn update(&mut self, inputs: &InputSnapshot, sounds: &mut SoundManager) -> SceneResult {
        if inputs.cancel_clicked {
            return SceneResult::Pop;
        }

        if inputs.menu_down_clicked {
            self.selected = (self.selected + 1) % self.buttons.len();
        }
        if inputs.menu_up_clicked {
            self.selected = ((self.selected + self.buttons.len()) - 1) % self.buttons.len();
        }

        self.cursor.update(inputs);

        for (i, button) in self.buttons.iter_mut().enumerate() {
            let selected = i == self.selected;
            if let Some(action) = button.update(selected, inputs, sounds) {
                if let Some(path) = action.strip_prefix("levelselect:") {
                    return SceneResult::PushLevelSelect {
                        path: Path::new(path).to_owned(),
                    };
                } else if let Some(path) = action.strip_prefix("level:") {
                    return SceneResult::PushLevel {
                        path: Path::new(path).to_owned(),
                    };
                } else if let Some(path) = action.strip_prefix("menu:") {
                    return SceneResult::PushMenu {
                        path: Path::new(path).to_owned(),
                    };
                } else {
                    error!("invalid button action: {action}");
                }
            }
        }

        SceneResult::Continue
    }

    fn draw(&mut self, context: &mut RenderContext, font: &Font) {
        context.player_batch.fill_rect(
            context.logical_area_in_subpixels(),
            Color {
                r: 0x33,
                g: 0x00,
                b: 0x33,
                a: 0xff,
            },
        );

        self.tilemap.draw_background(
            context,
            RenderLayer::Hud,
            context.logical_area_in_subpixels(),
            Point::zero(),
            &self.switches,
        );
        self.tilemap.draw_foreground(
            context,
            RenderLayer::Hud,
            context.logical_area_in_subpixels(),
            Point::zero(),
            &self.switches,
        );

        for button in self.buttons.iter() {
            button.draw(context, RenderLayer::Hud, font);
        }
        self.cursor.draw(context, RenderLayer::Hud);
    }
}
