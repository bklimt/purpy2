use std::path::PathBuf;

use crate::imagemanager::ImageManager;
use crate::inputmanager::InputSnapshot;
use crate::rendercontext::{RenderContext, RenderLayer};
use crate::scene::{Scene, SceneResult};
use crate::soundmanager::SoundManager;
use crate::utils::Color;

pub struct KillScreen {
    previous: Box<dyn Scene>,
    next: PathBuf,
}

impl<'a> KillScreen {
    pub fn new(previous: Box<dyn Scene>, next: PathBuf) -> KillScreen {
        KillScreen { previous, next }
    }
}

impl Scene for KillScreen {
    fn draw(&mut self, context: &mut RenderContext, images: &ImageManager) {
        let dest = context.logical_area_in_subpixels();
        self.previous.draw(context, images);

        let red_color = Color {
            r: 255,
            g: 0,
            b: 0,
            a: 127,
        };
        context.fill_rect(dest, RenderLayer::Hud, red_color);

        let text = "DEAD";
        let text_pos = (
            dest.w / 2 - text.len() as i32 * (images.font().char_width / 2),
            dest.h / 2 - text.len() as i32 * (images.font().char_height / 2),
        );
        images
            .font()
            .draw_string(context, RenderLayer::Hud, text_pos.into(), text);
    }

    fn update<'b, 'c>(
        &mut self,
        inputs: &'b InputSnapshot,
        _sounds: &'c mut SoundManager,
    ) -> SceneResult {
        if inputs.ok {
            SceneResult::SwitchToLevel {
                path: self.next.clone(),
            }
        } else if inputs.cancel {
            SceneResult::Pop
        } else {
            SceneResult::Continue
        }
    }
}
