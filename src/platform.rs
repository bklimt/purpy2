use anyhow::{Context, Result};

use crate::soundmanager::SoundManager;
use crate::sprite::SpriteBatch;
use crate::switchstate::SwitchState;
use crate::tilemap::MapObject;
use crate::tileset::{TileIndex, TileSet};
use crate::utils::{try_move_to_bounds, Direction, Point, Rect};

trait Platform {
    fn update(&mut self, switches: &SwitchState, sounds: &SoundManager);
    fn draw(&self, batch: &mut SpriteBatch, offset: Point);
    fn try_move_to(&self, player_rect: Rect, direction: Direction, is_backwards: bool);
}

struct PlatformBase<'a> {
    id: i32,
    tileset: &'a TileSet<'a>,
    tile_id: TileIndex,
    position: Rect,
    dx: i32,
    dy: i32,
    is_solid: bool,
    occupied: bool,
}

impl<'a> PlatformBase<'a> {
    fn new<'b>(obj: MapObject, tileset: &'b TileSet<'b>) -> Result<PlatformBase<'b>> {
        Ok(PlatformBase {
            id: obj.id,
            tileset: tileset,
            tile_id: obj.gid.context("gid required for platforms")? as TileIndex - 1,
            position: obj.position,
            dx: 0,
            dy: 0,
            is_solid: obj.solid,
            occupied: false,
        })
    }

    fn draw(&self, batch: &mut SpriteBatch, offset: Point) {
        let x = self.position.x + offset.x();
        let y = self.position.y + offset.y();
        let dest = self.position;
        if let Some(anim) = self.tileset.animations.get(&self.tile_id) {
            anim.blit(batch, dest, false);
        } else {
            let src = self.tileset.get_source_rect(self.tile_id);
            batch.draw(&self.tileset.sprite, Some(dest), Some(src));
        }
    }

    fn try_move_to(&self, player_rect: Rect, direction: Direction, is_backwards: bool) -> i32 {
        let area = if self.is_solid {
            self.position
        } else {
            if !matches!(direction, Direction::Down) {
                return 0;
            }
            if is_backwards {
                return 0;
            }
            Rect {
                x: self.position.x,
                y: self.position.y,
                w: self.position.w,
                h: self.position.h / 2,
            }
        };
        try_move_to_bounds(player_rect, area, direction)
    }
}

/*



class MovingPlatform(PlatformBase):
    direction: Direction
    distance: int
    start_x: int
    start_y: int
    end_x: int
    end_y: int
    moving_forward: bool
    condition: str | None
    overflow: str

    def __init__(self, obj: MapObject, tileset: TileSet):
        super().__init__(obj, tileset)
        self.distance = assert_int(
            obj.properties.get('distance', 0)) * SUBPIXELS
        # This is 16 for historical reasons, just because that's what the speed is tuned for.
        self.speed = (assert_int(
            obj.properties.get('speed', 1)) * SUBPIXELS) // 16
        self.start_x = self.x
        self.start_y = self.y
        self.moving_forward = True
        cond = obj.properties.get('condition')
        self.condition = assert_str(cond) if cond is not None else None
        self.overflow = assert_str(obj.properties.get('overflow', 'oscillate'))
        d = str(obj.properties.get('direction', 'N')).upper()
        if d == 'N':
            self.direction = Direction.UP
            self.distance *= self.tileset.tileheight
            self.end_x = self.start_x
            self.end_y = self.start_y - self.distance
        elif d == 'S':
            self.direction = Direction.DOWN
            self.distance *= self.tileset.tileheight
            self.end_x = self.start_x
            self.end_y = self.start_y + self.distance
        elif d == 'E':
            self.direction = Direction.RIGHT
            self.distance *= self.tileset.tilewidth
            self.end_x = self.start_x + self.distance
            self.end_y = self.start_y
        elif d == 'W':
            self.direction = Direction.LEFT
            self.distance *= self.tileset.tilewidth
            self.end_x = self.start_x - self.distance
            self.end_y = self.start_y
        else:
            raise Exception(f'unknown direction {d}')

        self.dx = 0
        self.dy = 0

    def update(self, switches: SwitchState, sounds: SoundManager):
        if self.condition != None:
            if not switches.is_condition_true(self.condition):
                self.moving_forward = False
                if self.x == self.start_x and self.y == self.start_y:
                    self.dx = 0
                    self.dy = 0
                    return

        self.dx = sign(self.end_x - self.start_x) * self.speed
        self.dy = sign(self.end_y - self.start_y) * self.speed
        if self.moving_forward:
            if self.direction == Direction.UP:
                if self.y <= self.end_y:
                    if self.overflow == 'wrap':
                        self.y += self.distance
                    elif self.overflow == 'clamp':
                        self.dy = 0
                        self.y = self.end_y + 1
                    else:
                        self.dy *= -1
                        self.moving_forward = False
            elif self.direction == Direction.DOWN:
                if self.y >= self.end_y:
                    if self.overflow == 'wrap':
                        self.y = self.start_y + (self.end_y - self.y)
                    elif self.overflow == 'clamp':
                        self.dy = 0
                        self.y = self.end_y - 1
                    else:
                        self.dy *= -1
                        self.moving_forward = False
            if self.direction == Direction.LEFT:
                if self.x <= self.end_x:
                    if self.overflow == 'wrap':
                        self.x += self.distance
                    elif self.overflow == 'clamp':
                        self.dx = 0
                        self.x = self.end_x + 1
                    else:
                        self.dx *= -1
                        self.moving_forward = False
            elif self.direction == Direction.RIGHT:
                if self.x >= self.end_x:
                    if self.overflow == 'wrap':
                        self.x = self.start_x + (self.end_x - self.x)
                    elif self.overflow == 'clamp':
                        self.dx = 0
                        self.x = self.end_x - 1
                    else:
                        self.dx *= -1
                        self.moving_forward = False
        else:
            # If must be oscillating.
            if self.direction == Direction.UP and self.y >= self.start_y:
                self.moving_forward = True
            elif self.direction == Direction.DOWN and self.y <= self.start_y:
                self.moving_forward = True
            elif self.direction == Direction.LEFT and self.x >= self.start_x:
                self.moving_forward = True
            elif self.direction == Direction.RIGHT and self.x <= self.start_x:
                self.moving_forward = True
            else:
                self.dx *= -1
                self.dy *= -1

        self.x += self.dx
        self.y += self.dy


class Bagel(PlatformBase):
    original_y: int
    falling: bool = False
    remaining: int = BAGEL_WAIT_TIME

    def __init__(self, obj: MapObject, tileset: TileSet):
        super().__init__(obj, tileset)
        self.original_y = self.y

    def draw(self, context: RenderContext, batch: SpriteBatch, offset: tuple[int, int]):
        x = self.x + offset[0]
        y = self.y + offset[1]
        area = self.tileset.get_source_rect(self.tile_id)
        if self.occupied:
            x += randint(-1, 1)
            y += randint(-1, 1)
        rect = pygame.Rect(x, y, area.w * SUBPIXELS, area.h * SUBPIXELS)
        if rect.bottom < 0 or rect.right < 0:
            return
        if rect.top >= context.logical_area.h or rect.right >= context.logical_area.w:
            return
        batch.draw(self.tileset.surface, rect, area)

    def update(self, switches: SwitchState, sounds: SoundManager):
        if self.falling:
            self.remaining -= 1
            if self.remaining == 0:
                self.dy = 0
                self.y = self.original_y
                self.falling = False
                self.remaining = BAGEL_WAIT_TIME
            else:
                self.dy += BAGEL_GRAVITY_ACCELERATION
                self.dy = max(self.dy, BAGEL_MAX_GRAVITY)
                self.y += self.dy
        else:
            if self.occupied:
                self.remaining -= 1
                if self.remaining == 0:
                    self.falling = True
                    self.remaining = BAGEL_FALL_TIME
                    self.dy = 0
            else:
                self.remaining = BAGEL_WAIT_TIME


class Conveyor(PlatformBase):
    def __init__(self, obj: MapObject, tileset: TileSet):
        super().__init__(obj, tileset)
        # This is hand-tuned.
        speed = int(obj.properties.get('speed', 24)) * SUBPIXELS//16
        if obj.properties.get('convey', 'E') == 'E':
            self.dx = speed
        else:
            self.dx = -1 * speed

    def update(self, switches: SwitchState, sounds: SoundManager):
        pass


class Spring(PlatformBase):
    sprite: SpriteSheet
    up: bool = False
    position: int = 0
    stall_counter = SPRING_STALL_FRAMES

    def __init__(self, obj: MapObject, tileset: TileSet, images: ImageManager):
        super().__init__(obj, tileset)
        surface = images.load_image('assets/sprites/spring.png')
        self.sprite = SpriteSheet(surface, 8, 8)

    @property
    def frame(self):
        return self.position // SUBPIXELS

    def draw(self, context: RenderContext, batch: SpriteBatch, offset: tuple[int, int]):
        x = self.x + offset[0]
        y = self.y + offset[1]
        dest = pygame.Rect(x, y, self.width, self.height)
        self.sprite.blit(batch, dest, self.frame)

    def should_boost(self):
        return self.up or (self.frame == SPRING_STEPS - 1)

    def update(self, switches: SwitchState, sounds: SoundManager):
        self.dx = 0
        self.dy = 0
        self.launch = False
        if not self.occupied:
            self.stall_counter = SPRING_STALL_FRAMES
            self.up = False
            if self.position > 0:
                self.position -= SPRING_SPEED
                self.dy = -SPRING_SPEED
        else:
            if self.up:
                self.stall_counter = SPRING_STALL_FRAMES
                if self.position > 0:
                    self.position -= SPRING_SPEED
                    self.dy = -SPRING_SPEED
                else:
                    self.launch = True
            else:
                if self.position < (SPRING_STEPS * SUBPIXELS) - SPRING_SPEED:
                    self.stall_counter = SPRING_STALL_FRAMES
                    self.position += SPRING_SPEED
                    self.dy = SPRING_SPEED
                else:
                    if self.stall_counter > 0:
                        self.stall_counter -= 1
                    else:
                        self.stall_counter = SPRING_STALL_FRAMES
                        self.up = True

    def try_move_to(self, player_rect: pygame.Rect, direction: Direction, is_backwards: bool) -> int:
        if self.is_solid:
            area = pygame.Rect(
                self.x,
                self.y + self.position,
                self.width,
                self.height - self.position)
            return try_move_to_bounds(player_rect, area, direction)
        else:
            if direction != Direction.DOWN:
                return 0
            if is_backwards:
                return 0
            area = pygame.Rect(
                self.x,
                self.y + self.position,
                self.width,
                self.height//2)
            return try_move_to_bounds(player_rect, area, direction)
*/
