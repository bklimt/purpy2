use std::ops::{self, Sub};

// Pixels

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pixels(i32);

impl From<i32> for Pixels {
    #[inline]
    fn from(value: i32) -> Self {
        Pixels(value)
    }
}

impl ops::Add<Pixels> for Pixels {
    type Output = Pixels;

    #[inline]
    fn add(self, rhs: Pixels) -> Self::Output {
        Pixels(self.0 + rhs.0)
    }
}

impl ops::Mul<i32> for Pixels {
    type Output = Pixels;

    #[inline]
    fn mul(self, rhs: i32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

// Subpixels

pub const SUBPIXELS: i32 = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Subpixels(i32);

impl Subpixels {
    // Returns self scaled and truncated to the pixel domain.
    #[inline]
    pub fn as_pixels(&self) -> Pixels {
        Pixels(self.0 / SUBPIXELS)
    }
}

impl From<Pixels> for Subpixels {
    #[inline]
    fn from(value: Pixels) -> Self {
        Self(value.0 * SUBPIXELS)
    }
}

impl From<i32> for Subpixels {
    #[inline]
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl ops::Add<Subpixels> for Subpixels {
    type Output = Subpixels;

    #[inline]
    fn add(self, rhs: Subpixels) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl ops::Mul<i32> for Subpixels {
    type Output = Subpixels;

    #[inline]
    fn mul(self, rhs: i32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

// Points

#[derive(Debug, Clone, Copy)]
struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    #[inline]
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl From<Point<Pixels>> for Point<Subpixels> {
    #[inline]
    fn from(value: Point<Pixels>) -> Self {
        Self::new(value.x.into(), value.y.into())
    }
}

impl Point<Subpixels> {
    #[inline]
    fn as_pixels(&self) -> Point<Pixels> {
        Point::new(self.x.as_pixels(), self.y.as_pixels())
    }
}

impl<T, U> From<(U, U)> for Point<T>
where
    U: Into<T>,
{
    #[inline]
    fn from(value: (U, U)) -> Self {
        Point::new(value.0.into(), value.1.into())
    }
}

impl<T> ops::Add<Point<T>> for Point<T>
where
    T: ops::Add<T, Output = T>,
{
    type Output = Point<T>;

    #[inline]
    fn add(self, rhs: Point<T>) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl<T, U> ops::Mul<U> for Point<T>
where
    T: ops::Mul<U, Output = T>,
    U: Copy,
{
    type Output = Point<T>;

    #[inline]
    fn mul(self, rhs: U) -> Self::Output {
        Point::new(self.x * rhs, self.y * rhs)
    }
}

// Rect

#[derive(Debug, Clone, Copy)]
struct Rect<T> {
    pub x: T,
    pub y: T,
    pub w: T,
    pub h: T,
}

impl<T> Rect<T>
where
    T: ops::Add<T, Output = T>,
{
    #[inline]
    pub fn new(x: T, y: T, w: T, h: T) -> Self {
        Self { x, y, w, h }
    }

    #[inline]
    pub fn top(self) -> T {
        self.y
    }
    #[inline]
    pub fn left(self) -> T {
        self.x
    }
    #[inline]
    pub fn right(self) -> T {
        self.x + self.w
    }
    #[inline]
    pub fn bottom(self) -> T {
        self.y + self.h
    }
}

impl<T> ops::Add<Point<T>> for Rect<T>
where
    T: ops::Add<T, Output = T>,
{
    type Output = Self;

    #[inline]
    fn add(self, rhs: Point<T>) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            w: self.w,
            h: self.h,
        }
    }
}

impl Rect<Subpixels> {
    #[inline]
    fn as_pixels(&self) -> Rect<Pixels> {
        Rect {
            x: self.x.as_pixels(),
            y: self.y.as_pixels(),
            w: self.w.as_pixels(),
            h: self.h.as_pixels(),
        }
    }
}

impl From<Rect<Pixels>> for Rect<Subpixels> {
    #[inline]
    fn from(value: Rect<Pixels>) -> Self {
        Self {
            x: value.x.into(),
            y: value.y.into(),
            w: value.w.into(),
            h: value.h.into(),
        }
    }
}

/*
// TODO: Implement these as needed.

impl<T> Into<sdl2::rect::Rect> for Rect<T> {
    #[inline]
    fn into(self) -> sdl2::rect::Rect {
        sdl2::rect::Rect::new(self.x, self.y, self.w as u32, self.h as u32)
    }
}

impl<T> Into<Option<sdl2::rect::Rect>> for Rect<T> {
    fn into(self) -> Option<sdl2::rect::Rect> {
        Some(self.into())
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixels_to_subpixels() {
        let pixels: Pixels = Pixels(4);
        let subpixels: Subpixels = pixels.into();
        assert_eq!(subpixels.0, 128);
    }

    #[test]
    fn subpixels_to_pixels() {
        let subpixels: Subpixels = Subpixels(128);
        let pixels: Pixels = subpixels.as_pixels();
        assert_eq!(pixels.0, 4);
    }

    #[test]
    fn point_constructor() {
        let point1 = Point {
            x: Subpixels(1),
            y: Subpixels(2),
        };
        let point2 = Point::new(Subpixels(3), Subpixels(4));
        let point3: Point<Pixels> = (Pixels(5), Pixels(6)).into();
        let point4: Point<Pixels> = (7, 8).into();

        assert_eq!(point1.x, Subpixels(1));
        assert_eq!(point1.y, Subpixels(2));
        assert_eq!(point2.x, Subpixels(3));
        assert_eq!(point2.y, Subpixels(4));
        assert_eq!(point3.x, Pixels(5));
        assert_eq!(point3.y, Pixels(6));
        assert_eq!(point4.x, Pixels(7));
        assert_eq!(point4.y, Pixels(8));
    }

    #[test]
    fn point_conversion() {
        let pixels: Point<Pixels> = (1, 2).into();
        let subpixels: Point<Subpixels> = pixels.into();
        assert_eq!(subpixels.x, Subpixels(32));
        assert_eq!(subpixels.y, Subpixels(64));

        let subpixels: Point<Subpixels> = (Pixels(3), Pixels(4)).into();
        assert_eq!(subpixels.x, Subpixels(96));
        assert_eq!(subpixels.y, Subpixels(128));

        let pixels = subpixels.as_pixels();
        assert_eq!(pixels.x, Pixels(3));
        assert_eq!(pixels.y, Pixels(4));
    }

    #[test]
    fn point_addition() {
        let point1: Point<Pixels> = (1, 2).into();
        let point2: Point<Pixels> = (3, 4).into();
        let point3 = point1 + point2;
        assert_eq!(point3.x, 4.into());
        assert_eq!(point3.y, 6.into());

        let point1: Point<Subpixels> = point1.into();
        let point2: Point<Subpixels> = point2.into();
        let point3 = point1 + point2;
        assert_eq!(point3.x, 128.into());
        assert_eq!(point3.y, 192.into());
    }

    #[test]
    fn point_multiplication() {
        let point1: Point<Pixels> = (1, 2).into();
        let point2 = point1 * 5;
        assert_eq!(point2.x, 5.into());
        assert_eq!(point2.y, 10.into());

        let point1: Point<Subpixels> = point1.into();
        let point2 = point1 * 5;
        assert_eq!(point2.x, 160.into());
        assert_eq!(point2.y, 320.into());
    }

    #[test]
    fn rect_getters() {
        let r = Rect::new(10, 20, 3, 4);
        assert_eq!(r.x, 10);
        assert_eq!(r.y, 20);
        assert_eq!(r.w, 3);
        assert_eq!(r.h, 4);
        assert_eq!(r.left(), 10);
        assert_eq!(r.top(), 20);
        assert_eq!(r.right(), 13);
        assert_eq!(r.bottom(), 24);
    }

    #[test]
    fn rect_add_point() {
        let r = Rect::new(10, 20, 3, 4);
        let p = Point::new(100, 200);
        let r = r + p;
        assert_eq!(r.x, 110);
        assert_eq!(r.y, 220);
        assert_eq!(r.w, 3);
        assert_eq!(r.h, 4);
        assert_eq!(r.left(), 110);
        assert_eq!(r.top(), 220);
        assert_eq!(r.right(), 113);
        assert_eq!(r.bottom(), 224);
    }

    #[test]
    fn rect_pixels_to_subpixels() {
        let r: Rect<Pixels> = Rect::new(1.into(), 2.into(), 3.into(), 4.into());
        let r: Rect<Subpixels> = r.into();
        assert_eq!(r.x, Subpixels(32));
        assert_eq!(r.y, Subpixels(64));
        assert_eq!(r.w, Subpixels(96));
        assert_eq!(r.h, Subpixels(128));

        let r = r.as_pixels();
        assert_eq!(r.x, Pixels(1));
        assert_eq!(r.y, Pixels(2));
        assert_eq!(r.w, Pixels(3));
        assert_eq!(r.h, Pixels(4));
    }
}
