use std::ops::{Index, IndexMut, Mul, Neg};

use crate::Facet;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Axis {
    X = 0,
    Y = 1,
    Z = 2,
    W = 3,
}

impl Axis {
    pub const ALL: [Self; 4] = [Self::X, Self::Y, Self::Z, Self::W];

    pub const fn id(self) -> u8 {
        self as u8
    }

    pub const fn from_id(id: u8) -> Self {
        match id {
            0 => Axis::X,
            1 => Axis::Y,
            2 => Axis::Z,
            3 => Axis::W,
            _ => panic!("bad axis ID"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Sign {
    Pos = 0,
    Neg = 1,
}

impl Neg for Sign {
    type Output = Sign;

    fn neg(self) -> Self::Output {
        Self::from_id(self.id() ^ 1)
    }
}

impl<T: Neg<Output = T>> Mul<T> for Sign {
    type Output = T;

    fn mul(self, rhs: T) -> Self::Output {
        match self {
            Sign::Pos => rhs,
            Sign::Neg => -rhs,
        }
    }
}

impl Sign {
    pub const fn id(self) -> u8 {
        self as u8
    }

    pub const fn from_id(id: u8) -> Self {
        if id == 0 { Self::Pos } else { Self::Neg }
    }

    pub const fn to_i8(self) -> i8 {
        match self {
            Sign::Pos => 1,
            Sign::Neg => -1,
        }
    }

    pub const fn try_from_i8(coordinate: i8) -> Option<Self> {
        match coordinate {
            ..0 => Some(Self::Neg),
            0 => None,
            1.. => Some(Self::Pos),
        }
    }
}

/// Integer position in 4-dimensional space.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Pos4(pub [i8; 4]);

impl Index<Axis> for Pos4 {
    type Output = i8;

    fn index(&self, axis: Axis) -> &Self::Output {
        &self.0[axis as usize]
    }
}

impl IndexMut<Axis> for Pos4 {
    fn index_mut(&mut self, axis: Axis) -> &mut Self::Output {
        &mut self.0[axis as usize]
    }
}

impl Pos4 {
    /// Rotates a position from `from` to `to`.
    ///
    /// Returns `None` if `from` and `to` are on the same axis.
    #[must_use]
    pub fn rot(self, from: Facet, to: Facet) -> Option<Self> {
        (from.axis != to.axis).then(|| {
            let mut ret = self;
            let sign = from.sign * to.sign;
            ret[to.axis] = sign * self[from.axis];
            ret[from.axis] = -sign * self[to.axis];
            ret
        })
    }

    pub fn display_pos(self) -> [i8; 2] {
        let [x, y, z, w] = self.0;
        [x + z * 6, y + w * 6]
    }
}
