enum Direction {
    None,
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn opposite(&self) -> Direction {
        match self {
            Direction::None => panic!("cannot take the opposite of no direction"),
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Right => Direction::Left,
            Direction::Left => Direction::Right,
        }
    }
}

fn sign(n: i32) -> i32 {
    if n < 0 {
        -1
    } else if n > 0 {
        1
    } else {
        0
    }
}

fn cmp_in_direction(a: i32, b: i32, direction: &Direction) -> i32 {
    match direction {
        Direction::Up | Direction::Left => sign(b - a),
        _ => sign(a - b),
    }
}

struct Rect {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl Rect {
    fn top(&self) -> i32 {
        self.y
    }
    fn bottom(&self) -> i32 {
        self.y + self.h
    }
    fn left(&self) -> i32 {
        self.x
    }
    fn right(&self) -> i32 {
        self.x + self.w
    }
}

/*
 * Try to move the actor rect in direction by delta and see if it intersects target.
 *
 * Returns the maximum distance the actor can move.
 */
fn try_move_to_bounds(actor: &Rect, target: &Rect, direction: &Direction) -> i32 {
    if actor.bottom() <= target.top() {
        0
    } else if actor.top() >= target.bottom() {
        0
    } else if actor.right() <= target.left() {
        0
    } else if actor.left() >= target.right() {
        0
    } else {
        match direction {
            Direction::None => panic!("cannot try_move_to in no direction"),
            Direction::Up => target.bottom() - actor.top(),
            Direction::Down => target.top() - actor.bottom(),
            Direction::Right => target.left() - actor.right(),
            Direction::Left => target.right() - actor.left(),
        }
    }
}

/*
def try_move_to_slope_bounds(
        actor: pygame.Rect,
        target: pygame.Rect,
        left_y: int,
        right_y: int,
        direction: Direction) -> int:
    """Try to move the actor rect in direction by delta and see if it intersects target.

    Returns the maximum distance the actor can move.
    """

    if actor.bottom <= target.top:
        return 0
    if actor.top >= target.bottom:
        return 0
    if actor.right <= target.left:
        return 0
    if actor.left >= target.right:
        return 0

    if direction != Direction.DOWN:
        return 0

    target_y: int = actor.bottom
    actor_center_x = (actor.left + actor.right) // 2

    if actor_center_x < target.left:
        target_y = target.top + left_y
    elif actor_center_x > target.right:
        target_y = target.top + right_y
    else:
        x_offset = actor_center_x - target.x
        slope = (right_y - left_y) / target.w
        target_y = int(target.y + slope * x_offset + left_y)

        if False:
            print(f'center_x = {actor_center_x}')
            print(f'x_offset = {x_offset}')
            print(f'slope = {slope}')
            print(f'target_y = {target_y}')
            print(f'actor_bottom = {actor.bottom}')

    if target_y < actor.bottom:
        return target_y - actor.bottom
    else:
        return 0


def intersect(rect1: pygame.Rect, rect2: pygame.Rect) -> bool:
    if rect1.right < rect2.left:
        return False
    if rect1.left > rect2.right:
        return False
    if rect1.bottom < rect2.top:
        return False
    if rect1.top > rect2.bottom:
        return False
    return True


def inside(rect: pygame.Rect, point: tuple[int, int]) -> bool:
    if point[0] < rect.left or point[0] > rect.right:
        return False
    if point[1] < rect.top or point[1] > rect.bottom:
        return False
    return True


def load_properties(
        node: xml.etree.ElementTree.Element,
        properties: dict[str, str | int | bool] | None = None
) -> dict[str, str | int | bool]:
    if properties is None:
        properties = {}

    for pnode in node:
        if pnode.tag == 'properties' or pnode.tag == 'property':
            load_properties(pnode, properties)

    if node.tag == 'property':
        name = node.attrib['name']
        typ = node.attrib.get('type', 'str')
        val = node.attrib['value']
        if typ == "str":
            properties[name] = val
        elif typ == "int":
            properties[name] = int(val)
        elif typ == "bool":
            properties[name] = (val == 'true')
        else:
            raise Exception(f'unsupported property type {typ}')

    return properties


def assert_bool(val: bool | int | str) -> bool:
    if not isinstance(val, bool):
        raise Exception(f'expected bool, got {val}')
    return val


def assert_int(val: bool | int | str) -> int:
    if not isinstance(val, int):
        raise Exception(f'expected int, got {val}')
    return val


def assert_str(val: bool | int | str) -> str:
    if not isinstance(val, str):
        raise Exception(f'expected str, got {val}')
    return val
*/
