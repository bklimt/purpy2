use std::collections::HashSet;
use std::fs;
use std::ops::RangeInclusive;
use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};

use crate::rendercontext::{RenderContext, RenderLayer};
use crate::utils::Rect;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Sprite {
    pub id: usize,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Sprite {
    pub fn subview(&self, rect: Rect) -> Sprite {
        Sprite {
            id: self.id,
            x: rect.x as u32,
            y: rect.y as u32,
            width: rect.w as u32,
            height: rect.h as u32,
        }
    }
}

pub struct SpriteSheet {
    sprite: Sprite,
    sprite_width: u32,
    sprite_height: u32,
    columns: u32,
}

impl SpriteSheet {
    pub fn new(sprite: Sprite, sprite_width: u32, sprite_height: u32) -> Result<SpriteSheet> {
        let w = sprite.width;
        let columns = w / sprite_width;
        Ok(SpriteSheet {
            sprite,
            sprite_width,
            sprite_height,
            columns,
        })
    }

    fn source_area(&self, index: u32, layer: u32) -> Rect {
        let row = (index / self.columns) + layer;
        let column = index % self.columns;

        let w = self.sprite_width as i32;
        let h = self.sprite_height as i32;
        let x = column as i32 * w;
        let y = row as i32 * h;
        Rect { x, y, w, h }
    }

    pub fn blit(
        &self,
        context: &mut RenderContext,
        layer: RenderLayer,
        dest: Rect,
        index: u32,
        sprite_layer: u32,
        reverse: bool,
    ) {
        let source_area = self.source_area(index, sprite_layer);
        if reverse {
            context.draw_reversed(self.sprite, layer, dest, source_area);
        } else {
            context.draw(self.sprite, layer, dest, source_area);
        }
    }
}

pub struct Animation {
    spritesheet: SpriteSheet,
    frames: u32,
    frames_per_frame: u32,
}

impl Animation {
    pub fn new(sprite: Sprite, sprite_width: u32, sprite_height: u32) -> Result<Animation> {
        if sprite.height != sprite_height {
            bail!("animations can only have one row");
        }
        let w = sprite.width;
        let spritesheet = SpriteSheet::new(sprite, sprite_width, sprite_height)?;
        let frames = w / sprite_width;
        let frames_per_frame = 2;
        Ok(Animation {
            spritesheet,
            frames,
            frames_per_frame,
        })
    }

    pub fn blit(&self, context: &mut RenderContext, layer: RenderLayer, dest: Rect, reverse: bool) {
        let index = (context.frame / self.frames_per_frame) % self.frames;
        self.spritesheet
            .blit(context, layer, dest, index, 0, reverse)
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
    pub fn from_file(path: &Path) -> Result<AnimationStateMachine> {
        let s = fs::read_to_string(path).map_err(|e| {
            anyhow!(
                "unable to load animation state machine at {:?}: {}",
                path,
                e
            )
        })?;
        AnimationStateMachine::new(&s)
    }

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
