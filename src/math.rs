use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

/// Two-dimensional point or vector used by all public APIs.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    /// Horizontal coordinate.
    pub x: f32,
    /// Vertical coordinate.
    pub y: f32,
}

impl Point {
    /// Creates a point from its coordinates.
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub(crate) fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub(crate) fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    pub(crate) fn normalized_or_zero(self) -> Self {
        let length = self.length();
        if length == 0.0 {
            Self::default()
        } else {
            self * (1.0 / length)
        }
    }

    pub(crate) fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
}

impl Add for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign for Point {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Point {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl SubAssign for Point {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Mul<f32> for Point {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Neg for Point {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y)
    }
}

/// Rectangle used for deterministic streamline seed placement.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    /// Minimum x coordinate.
    pub left: f32,
    /// Minimum y coordinate.
    pub top: f32,
    /// Maximum x coordinate.
    pub right: f32,
    /// Maximum y coordinate.
    pub bottom: f32,
}

impl Viewport {
    /// Creates a viewport.
    pub const fn new(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    pub(crate) fn width(self) -> f32 {
        self.right - self.left
    }

    pub(crate) fn height(self) -> f32 {
        self.bottom - self.top
    }

    pub(crate) fn is_finite(self) -> bool {
        self.left.is_finite()
            && self.top.is_finite()
            && self.right.is_finite()
            && self.bottom.is_finite()
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new(0.0, 0.0, 1.0, 1.0)
    }
}
