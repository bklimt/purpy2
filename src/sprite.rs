use anyhow::{anyhow, Result};
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

struct SpriteSheet<'a> {
    surface: Sprite<'a>,
    reverse: Sprite<'a>,
    sprite_width: i32,
    sprite_height: i32,
    columns: i32,
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
    fn new<'b, 'c>(
        surface: Surface<'b>,
        sprite_width: i32,
        sprite_height: i32,
        texture_creator: &'c TextureCreator<Canvas<Window>>,
    ) -> Result<SpriteSheet<'b>>
    where
        'c: 'b,
    {
        let w = surface.width() as i32;
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

    fn sprite(&self, index: i32, layer: i32, reverse: bool) -> Rect {
        let row = (index / self.columns) + layer;
        let column = if reverse {
            (self.columns - 1) - (index % self.columns)
        } else {
            index % self.columns
        };

        let w = self.sprite_width;
        let h = self.sprite_height;
        let x = column * w;
        let y = row * h;
        Rect { x, y, w, h }
    }

    fn blit<T>(&self, batch: &mut SpriteBatch, dest: Rect, index: i32, layer: i32, reverse: bool)
    where
        T: RenderTarget,
    {
        let texture = if reverse {
            &self.reverse
        } else {
            &self.surface
        };
        let sprite = self.sprite(index, layer, reverse);
        batch.draw(&texture, Some(dest), Some(sprite));
    }
}

/*

class Animation:
    spritesheet: SpriteSheet
    index: int = 0
    frames: int
    frames_per_frame: int = 2
    timer: int

    def __init__(self, surface: pygame.Surface, sprite_width: int, sprite_height: int):
        self.spritesheet = SpriteSheet(surface, sprite_width, sprite_height)
        if surface.get_height() != sprite_height:
            raise Exception('animations can only have one row')
        self.frames = surface.get_width() // sprite_width
        self.timer = self.frames_per_frame

    def update(self):
        if self.timer == 0:
            self.index = (self.index + 1) % self.frames
            self.timer = self.frames_per_frame
        else:
            self.timer -= 1

    def blit(self, batch: SpriteBatch, dest: pygame.Rect, reverse: bool):
        self.spritesheet.blit(batch, dest, self.index, reverse)


class AnimationStateMachineRule:
    current_range: tuple[int, int] | None  # This is an inclusive range.
    current_state: str | None
    next_frame: int | typing.Callable[[int], int]

    def __init__(self, text: str, acceptable_states: set[str]):
        # e.g. 1-2, STATE: +
        text = text.strip()
        parts = text.split(':')
        if len(parts) != 2:
            raise Exception(
                f'invalid animation state machine rule (missing colon): {text}')
        antecedent_text = parts[0].strip()
        consequent_text = parts[1].strip()

        antecedent_parts = antecedent_text.split(',')
        if len(antecedent_parts) != 2:
            raise Exception(
                f'invalid animation state machine rule (missing comma): {text}')
        range_text = antecedent_parts[0].strip()
        current_state_text = antecedent_parts[1].strip()

        if range_text == '*':
            self.current_range = None
        else:
            if range_text.find('-') < 0:
                self.current_range = (int(range_text), int(range_text))
            else:
                range_parts = range_text.split('-')
                if len(range_parts) != 2:
                    raise Exception(
                        f'invalid animation state machine rule (missing dash): {text}')
                range_start_text = range_parts[0].strip()
                range_end_text = range_parts[1].strip()
                self.current_range = (
                    int(range_start_text), int(range_end_text))

        if current_state_text == '*':
            self.current_state = None
        else:
            if current_state_text not in acceptable_states:
                raise Exception(
                    f'invalid animation state machine rule (invalid state): {text}')
            self.current_state = current_state_text

        if consequent_text == '+':
            self.next_frame = lambda x: x + 1
        elif consequent_text == '-':
            self.next_frame = lambda x: x - 1
        elif consequent_text == '=':
            self.next_frame = lambda x: x
        else:
            self.next_frame = int(consequent_text)

    def matches(self, current_frame: int, current_state: str) -> bool:
        if self.current_range is not None:
            if current_frame < self.current_range[0]:
                return False
            if current_frame > self.current_range[1]:
                return False
        if self.current_state is not None:
            if current_state != self.current_state:
                return False
        return True

    def apply(self, current_frame) -> int:
        if isinstance(self.next_frame, int):
            return self.next_frame
        return self.next_frame(current_frame)


class AnimationStateMachine:
    rules: list[AnimationStateMachineRule]

    def __init__(self, text: str):
        self.rules = []
        states: set[str] = set()
        in_transitions = False
        for line in text.split('\n'):
            line = line.strip()
            if line == '':
                continue
            if line[0] == '#':
                continue
            if line == '[STATES]':
                in_transitions = False
            elif line == '[TRANSITIONS]':
                in_transitions = True
            elif not in_transitions:
                states.add(line)
            else:
                rule = AnimationStateMachineRule(line, states)
                self.rules.append(rule)

    def next_frame(self, current_frame: int, current_state: str) -> int:
        for rule in self.rules:
            if rule.matches(current_frame, current_state):
                return rule.apply(current_frame)
        raise Exception(
            f'unhandled state machine case: {current_frame}, {current_state}')
*/
