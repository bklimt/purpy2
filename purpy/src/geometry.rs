use std::{cmp::Ordering, fmt::Display, ops};

use num_traits::Zero;

// How many subpixels to use for game logic.
const SUBPIXELS: i32 = 32;

// Pixels

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct Pixels(i32);

impl Pixels {
    const ZERO: Pixels = Pixels(0);

    #[inline]
    pub const fn new(n: i32) -> Self {
        Self(n)
    }

    #[inline]
    pub const fn as_subpixels(&self) -> Subpixels {
        Subpixels(self.0 * SUBPIXELS)
    }
}

impl Zero for Pixels {
    #[inline]
    fn zero() -> Self {
        Self::ZERO
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.0 == 0
    }

    #[inline]
    fn set_zero(&mut self) {
        self.0 = 0;
    }
}

impl Display for Pixels {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}px", self.0)
    }
}

impl ops::Add<Pixels> for Pixels {
    type Output = Pixels;

    #[inline]
    fn add(self, rhs: Pixels) -> Self::Output {
        Pixels(self.0 + rhs.0)
    }
}

impl ops::AddAssign<Pixels> for Pixels {
    #[inline]
    fn add_assign(&mut self, rhs: Pixels) {
        self.0 += rhs.0
    }
}

impl ops::SubAssign<Pixels> for Pixels {
    #[inline]
    fn sub_assign(&mut self, rhs: Pixels) {
        self.0 -= rhs.0
    }
}

impl ops::Mul<i32> for Pixels {
    type Output = Pixels;

    #[inline]
    fn mul(self, rhs: i32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl ops::Div<i32> for Pixels {
    type Output = Pixels;

    #[inline]
    fn div(self, rhs: i32) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl ops::Div<Pixels> for Pixels {
    type Output = i32;

    #[inline]
    fn div(self, rhs: Pixels) -> Self::Output {
        self.0 / rhs.0
    }
}

// Subpixels

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Subpixels(i32);

impl Subpixels {
    const ZERO: Subpixels = Subpixels(0);

    #[inline]
    pub const fn new(n: i32) -> Self {
        Self(n)
    }

    #[inline]
    pub const fn from_pixels(n: i32) -> Subpixels {
        Pixels(n).as_subpixels()
    }

    // Returns self scaled and truncated to the pixel domain.
    #[inline]
    pub fn as_pixels(&self) -> Pixels {
        Pixels(self.0 / SUBPIXELS)
    }

    #[inline]
    pub fn sign(self) -> i32 {
        match self.0.cmp(&0) {
            Ordering::Less => -1,
            Ordering::Greater => 1,
            Ordering::Equal => 0,
        }
    }

    #[inline]
    pub fn abs(self) -> Self {
        Self(self.0.abs())
    }
}

impl Zero for Subpixels {
    #[inline]
    fn zero() -> Self {
        Self::ZERO
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.0 == 0
    }

    #[inline]
    fn set_zero(&mut self) {
        self.0 = 0;
    }
}

impl Display for Subpixels {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}spx", self.0)
    }
}

impl ops::Add<Subpixels> for Subpixels {
    type Output = Subpixels;

    #[inline]
    fn add(self, rhs: Subpixels) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl ops::AddAssign<Subpixels> for Subpixels {
    #[inline]
    fn add_assign(&mut self, rhs: Subpixels) {
        self.0 += rhs.0;
    }
}

impl ops::Sub<Subpixels> for Subpixels {
    type Output = Subpixels;

    #[inline]
    fn sub(self, rhs: Subpixels) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl ops::SubAssign<Subpixels> for Subpixels {
    #[inline]
    fn sub_assign(&mut self, rhs: Subpixels) {
        self.0 -= rhs.0
    }
}

impl ops::Mul<i32> for Subpixels {
    type Output = Subpixels;

    #[inline]
    fn mul(self, rhs: i32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl ops::MulAssign<i32> for Subpixels {
    #[inline]
    fn mul_assign(&mut self, rhs: i32) {
        self.0 *= rhs
    }
}

impl ops::Div<i32> for Subpixels {
    type Output = Subpixels;

    #[inline]
    fn div(self, rhs: i32) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl ops::Div<Subpixels> for Subpixels {
    type Output = i32;

    #[inline]
    fn div(self, rhs: Subpixels) -> Self::Output {
        self.0 / rhs.0
    }
}

// Points

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    #[inline]
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl Point<Subpixels> {
    #[inline]
    pub fn as_pixels(&self) -> Point<Pixels> {
        Point::new(self.x.as_pixels(), self.y.as_pixels())
    }
}

impl<T> Zero for Point<T>
where
    T: Zero,
{
    #[inline]
    fn zero() -> Self {
        Self::new(T::zero(), T::zero())
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.x.is_zero() && self.y.is_zero()
    }

    #[inline]
    fn set_zero(&mut self) {
        self.x = T::zero();
        self.y = T::zero();
    }
}

impl From<Point<Pixels>> for Point<Subpixels> {
    #[inline]
    fn from(value: Point<Pixels>) -> Self {
        Self::new(value.x.as_subpixels(), value.y.as_subpixels())
    }
}

impl<T> From<(T, T)> for Point<T> {
    #[inline]
    fn from(value: (T, T)) -> Self {
        Point::new(value.0, value.1)
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

impl<T> ops::AddAssign<Point<T>> for Point<T>
where
    T: ops::AddAssign<T>,
{
    #[inline]
    fn add_assign(&mut self, rhs: Point<T>) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<T> ops::Sub<Point<T>> for Point<T>
where
    T: ops::Sub<T, Output = T>,
{
    type Output = Point<T>;

    #[inline]
    fn sub(self, rhs: Point<T>) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl<T> ops::SubAssign<Point<T>> for Point<T>
where
    T: ops::SubAssign<T>,
{
    #[inline]
    fn sub_assign(&mut self, rhs: Point<T>) {
        self.x -= rhs.x;
        self.y -= rhs.y;
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
pub struct Rect<T> {
    pub x: T,
    pub y: T,
    pub w: T,
    pub h: T,
}

impl<T> Rect<T>
where
    T: ops::Add<T, Output = T> + Copy + PartialOrd,
{
    #[inline]
    pub fn top(&self) -> T {
        self.y
    }
    #[inline]
    pub fn left(&self) -> T {
        self.x
    }
    #[inline]
    pub fn right(&self) -> T {
        self.x + self.w
    }
    #[inline]
    pub fn bottom(&self) -> T {
        self.y + self.h
    }
    #[inline]
    pub fn top_left(&self) -> Point<T> {
        Point::new(self.x, self.y)
    }

    pub fn intersects(&self, other: Rect<T>) -> bool {
        self.right() >= other.left()
            && self.left() <= other.right()
            && self.bottom() >= other.top()
            && self.top() <= other.bottom()
    }

    pub fn contains(&self, point: Point<T>) -> bool {
        point.x >= self.left()
            && point.x <= self.right()
            && point.y >= self.top()
            && point.y <= self.bottom()
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

impl<T> ops::AddAssign<Point<T>> for Rect<T>
where
    T: ops::AddAssign<T>,
{
    #[inline]
    fn add_assign(&mut self, rhs: Point<T>) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Rect<Subpixels> {
    #[inline]
    pub fn as_pixels(&self) -> Rect<Pixels> {
        Rect {
            x: self.x.as_pixels(),
            y: self.y.as_pixels(),
            w: self.w.as_pixels(),
            h: self.h.as_pixels(),
        }
    }
}

impl Rect<Pixels> {
    #[inline]
    pub fn as_subpixels(&self) -> Rect<Subpixels> {
        Rect {
            x: self.x.as_subpixels(),
            y: self.y.as_subpixels(),
            w: self.w.as_subpixels(),
            h: self.h.as_subpixels(),
        }
    }
}

impl From<Rect<Pixels>> for Rect<Subpixels> {
    #[inline]
    fn from(value: Rect<Pixels>) -> Self {
        value.as_subpixels()
    }
}

#[cfg(feature = "sdl2")]
impl From<Rect<Pixels>> for sdl2::rect::Rect {
    #[inline]
    fn from(value: Rect<Pixels>) -> Self {
        sdl2::rect::Rect::new(value.x.0, value.y.0, value.w.0 as u32, value.h.0 as u32)
    }
}

#[cfg(feature = "sdl2")]
impl From<Rect<Pixels>> for Option<sdl2::rect::Rect> {
    #[inline]
    fn from(value: Rect<Pixels>) -> Self {
        Some(value.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixels_to_subpixels() {
        let pixels: Pixels = Pixels(4);
        let subpixels: Subpixels = pixels.as_subpixels();
        assert_eq!(subpixels.0, 128);

        let subpixels = pixels.as_subpixels();
        assert_eq!(subpixels.0, 128);
    }

    #[test]
    fn subpixels_to_pixels() {
        let subpixels: Subpixels = Subpixels(128);
        let pixels: Pixels = subpixels.as_pixels();
        assert_eq!(pixels.0, 4);
    }

    #[test]
    fn subpixels_math() {
        let x1 = Subpixels(32);
        let x2 = x1 + Subpixels(12);
        assert_eq!(x2, Subpixels(44));

        let x3 = x2 - Subpixels(12);
        assert_eq!(x3, Subpixels(32));
    }

    #[test]
    fn point_constructor() {
        let point1 = Point {
            x: Subpixels(1),
            y: Subpixels(2),
        };
        let point2 = Point::new(Subpixels(3), Subpixels(4));
        let point3: Point<Pixels> = (Pixels(5), Pixels(6)).into();
        let point4: Point<Pixels> = (Pixels(7), Pixels(8)).into();

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
        let pixels: Point<Pixels> = (Pixels(1), Pixels(2)).into();
        let subpixels: Point<Subpixels> = pixels.into();
        assert_eq!(subpixels.x, Subpixels(32));
        assert_eq!(subpixels.y, Subpixels(64));

        let subpixels: Point<Subpixels> =
            (Pixels(3).as_subpixels(), Pixels(4).as_subpixels()).into();
        assert_eq!(subpixels.x, Subpixels(96));
        assert_eq!(subpixels.y, Subpixels(128));
    }

    #[test]
    fn point_addition() {
        let point1: Point<Pixels> = (Pixels(1), Pixels(2)).into();
        let point2: Point<Pixels> = (Pixels(3), Pixels(4)).into();
        let point3 = point1 + point2;
        assert_eq!(point3.x, Pixels(4));
        assert_eq!(point3.y, Pixels(6));

        let point1: Point<Subpixels> = point1.into();
        let point2: Point<Subpixels> = point2.into();
        let point3 = point1 + point2;
        assert_eq!(point3.x, Subpixels::new(128));
        assert_eq!(point3.y, Subpixels::new(192));
    }

    #[test]
    fn point_multiplication() {
        let point1: Point<Pixels> = (Pixels::new(1), Pixels::new(2)).into();
        let point2 = point1 * 5;
        assert_eq!(point2.x, Pixels(5));
        assert_eq!(point2.y, Pixels(10));

        let point1: Point<Subpixels> = point1.into();
        let point2 = point1 * 5;
        assert_eq!(point2.x, Subpixels::new(160));
        assert_eq!(point2.y, Subpixels::new(320));
    }

    #[test]
    fn rect_getters() {
        let r = Rect {
            x: 10,
            y: 20,
            w: 3,
            h: 4,
        };
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
        let r = Rect {
            x: 10,
            y: 20,
            w: 3,
            h: 4,
        };
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
        let r: Rect<Pixels> = Rect {
            x: Pixels(1),
            y: Pixels(2),
            w: Pixels(3),
            h: Pixels(4),
        };
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
