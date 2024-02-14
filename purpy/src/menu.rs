use std::path::{Path, PathBuf};

use anyhow::Result;
use log::error;
use num_traits::Zero;

use crate::cursor::Cursor;
use crate::filemanager::FileManager;
use crate::font::Font;
use crate::geometry::{Point, Subpixels};
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
    reload_path: Option<PathBuf>,
    cursor: Cursor,
    tilemap: TileMap,
    buttons: Vec<UiButton>,
    horizontal_button_order: Vec<usize>,
    vertical_button_order: Vec<usize>,
    selected: usize,
    switches: SwitchState,
}

enum ButtonOrderDirection {
    Vertical,
    Horizontal,
}

impl Menu {
    pub fn new_menu(
        path: &Path,
        files: &FileManager,
        images: &mut dyn ImageLoader,
    ) -> Result<Self> {
        Self::new(path, None, files, images)
    }

    pub fn new_death_screen(
        level_path: PathBuf,
        files: &FileManager,
        images: &mut dyn ImageLoader,
    ) -> Result<Self> {
        let path = Path::new("assets/menus/dead.tmx");
        Self::new(path, Some(level_path), files, images)
    }

    pub fn new_pause_screen(
        level_path: PathBuf,
        files: &FileManager,
        images: &mut dyn ImageLoader,
    ) -> Result<Self> {
        let path = Path::new("assets/menus/pause.tmx");
        Self::new(path, Some(level_path), files, images)
    }

    fn new(
        path: &Path,
        reload_path: Option<PathBuf>,
        files: &FileManager,
        images: &mut dyn ImageLoader,
    ) -> Result<Self> {
        let cursor = Cursor::new(images)?;
        let tilemap = TileMap::from_file(path, files, images)?;
        let mut buttons = Vec::new();
        let switches = SwitchState::new();

        for obj in &tilemap.objects {
            if obj.properties.uibutton.is_some() {
                buttons.push(UiButton::new(
                    obj,
                    tilemap.tilewidth,
                    tilemap.tileheight,
                    images,
                )?);
            }
        }

        let mut button_positions: Vec<(usize, Subpixels, Subpixels)> = buttons
            .iter()
            .enumerate()
            .map(|(i, button)| (i, button.position.x, button.position.y))
            .collect();

        button_positions.sort_by_key(|(_, x, y)| (*y, *x));
        let horizontal_button_order: Vec<usize> =
            button_positions.iter().map(|(i, _, _)| *i).collect();

        button_positions.sort_by_key(|(_, x, y)| (*x, *y));
        let vertical_button_order: Vec<usize> =
            button_positions.iter().map(|(i, _, _)| *i).collect();

        let selected = vertical_button_order[0];

        Ok(Self {
            reload_path,
            cursor,
            tilemap,
            buttons,
            vertical_button_order,
            horizontal_button_order,
            selected,
            switches,
        })
    }

    fn next_button(&mut self, delta: i32, direction: ButtonOrderDirection) {
        let order: &[usize] = match direction {
            ButtonOrderDirection::Horizontal => &self.horizontal_button_order,
            ButtonOrderDirection::Vertical => &self.vertical_button_order,
        };
        let Some(pos) = order.iter().position(|i| *i == self.selected) else {
            error!("invalid button index: {}", self.selected);
            return;
        };
        let new_pos = ((pos + order.len()) as i32 + delta) as usize % order.len();
        self.selected = order[new_pos];
    }
}

impl Scene for Menu {
    fn update(
        &mut self,
        _context: &RenderContext,
        inputs: &InputSnapshot,
        sounds: &mut SoundManager,
    ) -> SceneResult {
        if inputs.cancel_clicked {
            return SceneResult::Pop;
        }

        if inputs.menu_down_clicked {
            self.next_button(1, ButtonOrderDirection::Vertical);
        }
        if inputs.menu_up_clicked {
            self.next_button(-1, ButtonOrderDirection::Vertical);
        }
        if inputs.menu_left_clicked {
            self.next_button(-1, ButtonOrderDirection::Horizontal);
        }
        if inputs.menu_right_clicked {
            self.next_button(1, ButtonOrderDirection::Horizontal);
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
                } else if action == "pop" {
                    return SceneResult::Pop;
                } else if action == "pop2" {
                    return SceneResult::PopTwo;
                } else if action == "reload" {
                    if let Some(reload_path) = &self.reload_path {
                        return SceneResult::ReloadLevel {
                            path: reload_path.clone(),
                        };
                    } else {
                        error!("menu button triggered reload, but no reload_path set");
                    }
                } else {
                    error!("invalid button action: {action}");
                }
            }
        }

        SceneResult::Continue
    }

    fn draw(&self, context: &mut RenderContext, font: &Font, previous: Option<&dyn Scene>) {
        context.player_batch.fill_rect(
            context.logical_area_in_subpixels(),
            Color {
                r: 0x33,
                g: 0x00,
                b: 0x33,
                a: 0xff,
            },
        );

        if let Some(background) = previous {
            background.draw(context, font, None);
        }

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
