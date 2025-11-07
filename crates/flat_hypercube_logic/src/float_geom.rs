use std::{
    f32::consts::FRAC_PI_2,
    ops::{Index, IndexMut},
};

use crate::{Axis, Facet, Pos4};

/// Floating-point position in 4-dimensional space.
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct FloatPos4([f32; 4]);

impl From<Pos4> for FloatPos4 {
    fn from(pos: Pos4) -> Self {
        Self(pos.0.map(|coord| coord as f32))
    }
}

impl From<FloatPos4> for Pos4 {
    fn from(pos: FloatPos4) -> Self {
        Self(pos.0.map(|coord| coord.round() as i8))
    }
}

impl Index<Axis> for FloatPos4 {
    type Output = f32;

    fn index(&self, axis: Axis) -> &Self::Output {
        &self.0[axis as usize]
    }
}

impl IndexMut<Axis> for FloatPos4 {
    fn index_mut(&mut self, axis: Axis) -> &mut Self::Output {
        &mut self.0[axis as usize]
    }
}

impl FloatPos4 {
    /// Rotates a position from `from` to `to` by `t` quarters of a full
    /// rotation.
    ///
    /// Returns `None` if `from` and `to` are on the same axis.
    #[must_use]
    pub fn rot(self, from: Facet, to: Facet, t: f32) -> Option<Self> {
        (from.axis != to.axis).then(|| {
            let mut ret = self;
            let sign = from.sign * to.sign;
            let (s, c) = (sign * t * FRAC_PI_2).sin_cos();
            ret[to.axis] = c * self[to.axis] + s * self[from.axis];
            ret[from.axis] = c * self[from.axis] - s * self[to.axis];
            ret
        })
    }

    pub fn display_pos(self) -> [f32; 2] {
        let [x, y, z, w] = self.0;
        [x + z * 6.0, y + w * 6.0]
    }

    pub fn integer_display_pos(self) -> [i8; 2] {
        self.display_pos().map(|coord| coord.round() as i8)
    }
}
