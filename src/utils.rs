use std::collections::HashMap;

pub type Subpixels = i32;

pub struct Point(Subpixels, Subpixels);

#[derive(Debug)]
pub enum Direction {
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

fn sign(n: Subpixels) -> Subpixels {
    if n < 0 {
        -1
    } else if n > 0 {
        1
    } else {
        0
    }
}

fn cmp_in_direction(a: Subpixels, b: Subpixels, direction: Direction) -> Subpixels {
    match direction {
        Direction::Up | Direction::Left => sign(b - a),
        _ => sign(a - b),
    }
}

pub struct Rect {
    pub x: Subpixels,
    pub y: Subpixels,
    pub w: Subpixels,
    pub h: Subpixels,
}

impl Rect {
    pub fn top(&self) -> Subpixels {
        self.y
    }
    pub fn bottom(&self) -> Subpixels {
        self.y + self.h
    }
    pub fn left(&self) -> Subpixels {
        self.x
    }
    pub fn right(&self) -> Subpixels {
        self.x + self.w
    }
}

/*
 * Try to move the actor rect in direction by delta and see if it intersects target.
 *
 * Returns the maximum distance the actor can move.
 */
fn try_move_to_bounds(actor: Rect, target: Rect, direction: Direction) -> Subpixels {
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
 * Try to move the actor rect in direction by delta and see if it intersects target.
 *
 * Returns the maximum distance the actor can move.
 */
fn try_move_to_slope_bounds(
    actor: &Rect,
    target: &Rect,
    left_y: Subpixels,
    right_y: Subpixels,
    direction: &Direction,
) -> Subpixels {
    if actor.bottom() <= target.top() {
        return 0;
    }
    if actor.top() >= target.bottom() {
        return 0;
    }
    if actor.right() <= target.left() {
        return 0;
    }
    if actor.left() >= target.right() {
        return 0;
    }

    if let Direction::Down = direction {
        return 0;
    }

    let mut target_y = actor.bottom();
    let actor_center_x = (actor.left() + actor.right()) / 2;

    if actor_center_x < target.left() {
        target_y = target.top() + left_y;
    } else if actor_center_x > target.right() {
        target_y = target.top() + right_y;
    } else {
        let x_offset = actor_center_x - target.x;
        let slope = (right_y - left_y) / target.w;
        target_y = target.y + slope * x_offset + left_y;

        if false {
            println!("center_x = {actor_center_x}");
            println!("x_offset = {x_offset}");
            println!("slope = {slope}");
            println!("target_y = {target_y}");
            println!("actor_bottom = {}", actor.bottom());
        }
    }

    if target_y < actor.bottom() {
        target_y - actor.bottom()
    } else {
        0
    }
}

fn intersect(rect1: Rect, rect2: Rect) -> bool {
    if rect1.right() < rect2.left() {
        false
    } else if rect1.left() > rect2.right() {
        false
    } else if rect1.bottom() < rect2.top() {
        false
    } else if rect1.top() > rect2.bottom() {
        false
    } else {
        true
    }
}

fn inside(rect: Rect, point: Point) -> bool {
    if point.0 < rect.left() || point.0 > rect.right() {
        false
    } else if point.1 < rect.top() || point.1 > rect.bottom() {
        false
    } else {
        true
    }
}

pub enum PropertyValue {
    StringValue(String),
    IntValue(i32),
    BoolValue(bool),
}

pub type PropertyMap = HashMap<String, PropertyValue>;

/*
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
