use std::collections::HashSet;
use std::ops::RangeInclusive;

use anyhow::{anyhow, bail, Context, Result};
use sdl2::render::{Canvas, RenderTarget, Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::video::Window;

use crate::utils::Rect;

pub struct Sprite<'a> {
    surface: Surface<'a>,
    texture: Texture<'a>,
}

impl<'a> Sprite<'a> {
    pub fn new<'b, 'c, T>(
        surface: Surface<'b>,
        texture_creator: &'c TextureCreator<T>,
    ) -> Result<Sprite<'b>>
    where
        'c: 'b,
    {
        let texture = surface.as_texture(texture_creator)?;
        Ok(Sprite { surface, texture })
    }

    pub fn width(&self) -> u32 {
        self.surface.width()
    }

    pub fn height(&self) -> u32 {
        self.surface.width()
    }
}

pub struct SpriteBatch<'a> {
    canvas: &'a mut Canvas<Window>,
}

impl<'a> SpriteBatch<'a> {
    pub fn new<'b>(canvas: &'b mut Canvas<Window>) -> SpriteBatch<'b> {
        SpriteBatch { canvas }
    }

    pub fn draw(&mut self, sprite: &Sprite, dst: Option<Rect>, src: Option<Rect>) {
        let src = src.map(|r| r.into());
        let dst = dst.map(|r| r.into());
        self.canvas.copy(&sprite.texture, src, dst);
    }
}

pub struct SpriteSheet<'a> {
    surface: Sprite<'a>,
    reverse: Sprite<'a>,
    sprite_width: u32,
    sprite_height: u32,
    columns: u32,
}

fn reverse_surface(surface: &Surface) -> Result<Surface<'static>> {
    let w = surface.width();
    let h = surface.height();
    let pitch = surface.pitch();
    let format = surface.pixel_format_enum();

    let mut reverse = Surface::new(w, h, format).map_err(|s: String| anyhow!("{}", s))?;
    let reverse_pitch = reverse.pitch() as usize;

    let w = w as usize;
    let h = h as usize;
    let pitch = pitch as usize;

    reverse.with_lock_mut(|dst| {
        surface.with_lock(|src| {
            for x in 0..w {
                let dx = (w - 1) - x;
                for y in 0..h {
                    let sp = x + y * pitch;
                    let dp = dx + y * reverse_pitch;
                    dst[dp] = src[sp];
                }
            }
        });
    });

    Ok(reverse)
}

impl<'a> SpriteSheet<'a> {
    pub fn new<'b, 'c, T>(
        surface: Surface<'b>,
        sprite_width: u32,
        sprite_height: u32,
        texture_creator: &'c TextureCreator<T>,
    ) -> Result<SpriteSheet<'b>>
    where
        'c: 'b,
    {
        let w = surface.width();
        let reverse = reverse_surface(&surface)?;
        let surface = Sprite::new(surface, texture_creator)?;
        let reverse = Sprite::new(reverse, texture_creator)?;
        let columns = w / sprite_width;
        Ok(SpriteSheet {
            surface,
            reverse,
            sprite_width,
            sprite_height,
            columns,
        })
    }

    fn sprite(&self, index: u32, layer: u32, reverse: bool) -> Rect {
        let row = (index / self.columns) + layer;
        let column = if reverse {
            (self.columns - 1) - (index % self.columns)
        } else {
            index % self.columns
        };

        let w = self.sprite_width as i32;
        let h = self.sprite_height as i32;
        let x = column as i32 * w;
        let y = row as i32 * h;
        Rect { x, y, w, h }
    }

    pub fn blit(&self, batch: &mut SpriteBatch, dest: Rect, index: u32, layer: u32, reverse: bool) {
        let texture = if reverse {
            &self.reverse
        } else {
            &self.surface
        };
        let sprite = self.sprite(index, layer, reverse);
        batch.draw(&texture, Some(dest), Some(sprite));
    }
}

pub struct Animation<'a> {
    spritesheet: SpriteSheet<'a>,
    index: u32,
    frames: u32,
    frames_per_frame: u32,
    timer: u32,
}

impl<'a> Animation<'a> {
    pub fn new<'b, 'c, T>(
        surface: Surface<'b>,
        sprite_width: u32,
        sprite_height: u32,
        texture_creator: &'c TextureCreator<T>,
    ) -> Result<Animation<'b>>
    where
        'c: 'b,
    {
        if surface.height() != sprite_height {
            bail!("animations can only have one row");
        }
        let w = surface.width();
        let spritesheet = SpriteSheet::new(surface, sprite_width, sprite_height, texture_creator)?;
        let index = 0;
        let frames = w / sprite_width;
        let frames_per_frame = 2;
        let timer = frames_per_frame;
        Ok(Animation {
            spritesheet,
            index,
            frames,
            frames_per_frame,
            timer,
        })
    }

    pub fn update(&mut self) {
        if self.timer == 0 {
            self.index = (self.index + 1) % self.frames;
            self.timer = self.frames_per_frame;
        } else {
            self.timer -= 1;
        }
    }

    pub fn blit(&self, batch: &mut SpriteBatch, dest: Rect, reverse: bool) {
        self.spritesheet.blit(batch, dest, self.index, 0, reverse)
    }
}

enum NextFrame {
    Value(u32),
    Function(fn(u32) -> u32),
}

impl NextFrame {
    fn next(&self, frame: u32) -> u32 {
        match self {
            NextFrame::Value(n) => *n,
            NextFrame::Function(f) => f(frame),
        }
    }
}

struct AnimationStateMachineRule {
    current_range: Option<RangeInclusive<u32>>,
    current_state: Option<String>,
    next_frame: NextFrame,
}

impl AnimationStateMachineRule {
    fn new(text: &str, acceptable_states: &HashSet<String>) -> Result<AnimationStateMachineRule> {
        // e.g. 1-2, STATE: +
        let text = text.trim();
        let colon = text.find(':').context(format!(
            "invalid animation state machine rule (missing colon): {text}"
        ))?;
        let (antecedent, consequent) = text.split_at(colon);
        let antecedent = antecedent.trim();
        let consequent = consequent[1..].trim();

        let comma = antecedent.find(',').context(format!(
            "invalid animation state machine rule (missing comma): {text}"
        ))?;
        let (range, current_state) = antecedent.split_at(comma);
        let range = range.trim();
        let current_state = current_state[1..].trim();

        let current_range = if range == "*" {
            None
        } else {
            Some(match range.find('-') {
                None => {
                    let n = range.parse::<u32>()?;
                    n..=n
                }
                Some(dash) => {
                    let (range_start, range_end) = range.split_at(dash);
                    let range_start = range_start.trim();
                    let range_end = range_end[1..].trim();
                    let range_start = range_start
                        .parse::<u32>()
                        .map_err(|e| anyhow!("invalid number {}: {}", range_start, e))?;
                    let range_end = range_end
                        .parse::<u32>()
                        .map_err(|e| anyhow!("invalid number {}: {}", range_end, e))?;
                    range_start..=range_end
                }
            })
        };

        let current_state = if current_state == "*" {
            None
        } else {
            if !acceptable_states.contains(current_state) {
                bail!("invalid animation state machine rule (invalid state): {text}");
            }
            Some(current_state.to_owned())
        };

        let next_frame = match consequent {
            "+" => NextFrame::Function(|x| x + 1),
            "-" => NextFrame::Function(|x| x - 1),
            "=" => NextFrame::Function(|x| x),
            _ => NextFrame::Value(
                consequent
                    .parse()
                    .map_err(|e| anyhow!("invalid number {}: {}", consequent, e))?,
            ),
        };

        Ok(AnimationStateMachineRule {
            current_range,
            current_state,
            next_frame,
        })
    }

    fn matches(&self, current_frame: u32, current_state: &str) -> bool {
        if let Some(range) = &self.current_range {
            if !range.contains(&current_frame) {
                return false;
            }
        }
        if let Some(state) = &self.current_state {
            if current_state != state {
                return false;
            }
        }
        return true;
    }

    fn apply(&self, current_frame: u32) -> u32 {
        self.next_frame.next(current_frame)
    }
}

pub struct AnimationStateMachine {
    rules: Vec<AnimationStateMachineRule>,
}

impl AnimationStateMachine {
    pub fn new(text: &str) -> Result<AnimationStateMachine> {
        let mut rules = Vec::new();
        let mut states = HashSet::new();
        let mut in_transitions = false;
        for line in text.lines() {
            let line = line.trim();
            if line.len() == 0 {
                continue;
            }
            if line.starts_with('#') {
                continue;
            }
            if line == "[STATES]" {
                in_transitions = false;
            } else if line == "[TRANSITIONS]" {
                in_transitions = true;
            } else if !in_transitions {
                states.insert(line.to_owned());
            } else {
                let rule = AnimationStateMachineRule::new(line, &states)
                    .map_err(|e| anyhow!("invalid rule {}: {}", line, e))?;
                rules.push(rule);
            }
        }
        Ok(AnimationStateMachine { rules })
    }

    pub fn next_frame(&self, current_frame: u32, current_state: &str) -> Result<u32> {
        for rule in self.rules.iter() {
            if rule.matches(current_frame, current_state) {
                return Ok(rule.apply(current_frame));
            }
        }
        Err(anyhow!(
            "unhandled state machine case: {current_frame}, {current_state}"
        ))
    }
}
