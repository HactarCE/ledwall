use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};

/// Tetris block position. (0, 0) is the bottom-left corner.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Pos {
    pub x: i8,
    pub y: i8,
}

impl Pos {
    pub const fn new(x: i8, y: i8) -> Self {
        Self { x, y }
    }
}

/// Offset from a position.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Offset {
    pub dx: i8,
    pub dy: i8,
}

impl Offset {
    pub const ZERO: Self = Self::new(0, 0);
    pub const LEFT: Self = Self::new(-1, 0);
    pub const RIGHT: Self = Self::new(1, 0);
    pub const DOWN: Self = Self::new(0, -1);

    pub const fn new(dx: i8, dy: i8) -> Self {
        Self { dx, dy }
    }
}

impl Add for Offset {
    type Output = Offset;

    fn add(self, rhs: Self) -> Self::Output {
        Offset {
            dx: self.dx + rhs.dx,
            dy: self.dy + rhs.dy,
        }
    }
}

impl Sub for Offset {
    type Output = Offset;

    fn sub(self, rhs: Self) -> Self::Output {
        Offset {
            dx: self.dx - rhs.dx,
            dy: self.dy - rhs.dy,
        }
    }
}

impl AddAssign for Offset {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl SubAssign for Offset {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Neg for Offset {
    type Output = Offset;

    fn neg(self) -> Self::Output {
        Self {
            dx: -self.dx,
            dy: -self.dy,
        }
    }
}

impl Add<Offset> for Pos {
    type Output = Pos;

    fn add(self, rhs: Offset) -> Self::Output {
        Pos {
            x: self.x + rhs.dx,
            y: self.y + rhs.dy,
        }
    }
}

impl Sub<Offset> for Pos {
    type Output = Pos;

    fn sub(self, rhs: Offset) -> Self::Output {
        Pos {
            x: self.x - rhs.dx,
            y: self.y - rhs.dy,
        }
    }
}

impl AddAssign<Offset> for Pos {
    fn add_assign(&mut self, rhs: Offset) {
        *self = *self + rhs;
    }
}

impl SubAssign<Offset> for Pos {
    fn sub_assign(&mut self, rhs: Offset) {
        *self = *self - rhs;
    }
}
