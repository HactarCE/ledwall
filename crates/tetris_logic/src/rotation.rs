use crate::Offset;

/// Rotation state.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Rot {
    /// Initial state.
    #[default]
    Init = 0,
    /// Rotated 90° clockwise.
    Cw = 1,
    /// Rotated 180°.
    Double = 2,
    /// Rotated 90° counterclockwise.
    Ccw = 3,
}

impl Rot {
    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_index(index: usize) -> Self {
        match index & 0b11 {
            0 => Self::Init,
            1 => Self::Cw,
            2 => Self::Double,
            3 => Self::Ccw,
            _ => unreachable!(),
        }
    }

    #[must_use]
    pub fn rot_cw(self) -> Self {
        Self::from_index(self.index() + 1)
    }

    #[must_use]
    pub fn rot_180(self) -> Self {
        Self::from_index(self.index() + 2)
    }

    #[must_use]
    pub fn rot_ccw(self) -> Self {
        Self::from_index(self.index() + 3)
    }

    pub fn apply(self, offset: Offset) -> Offset {
        match self {
            Rot::Init => offset,
            Rot::Cw => Offset::new(offset.dy, -offset.dx),
            Rot::Double => -offset,
            Rot::Ccw => Offset::new(-offset.dy, offset.dx),
        }
    }
}
