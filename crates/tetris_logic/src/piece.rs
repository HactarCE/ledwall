use crate::{Offset, Pos, Rot};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Tetromino {
    I,
    J,
    L,
    O,
    S,
    T,
    Z,
}

impl Tetromino {
    /// Offsets for computing SRS kick tables, taken from [How Guideline SRS
    /// _Really_ Works][guideline-srs].
    ///
    /// [guideline-srs]:
    ///     https://tetris.wiki/Super_Rotation_System#How_Guideline_SRS_Really_Works
    fn offsets(self, rot: Rot) -> [Offset; 5] {
        match self {
            Tetromino::J | Tetromino::L | Tetromino::S | Tetromino::T | Tetromino::Z => match rot {
                Rot::Init => [(0, 0); 5],
                Rot::Cw => [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
                Rot::Double => [(0, 0); 5],
                Rot::Ccw => [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
            },
            Tetromino::I => match rot {
                Rot::Init => [(0, 0), (-1, 0), (2, 0), (-1, 0), (2, 0)],
                Rot::Cw => [(-1, 0), (0, 0), (0, 0), (0, 1), (0, -2)],
                Rot::Double => [(-1, 1), (1, 1), (-2, 1), (1, 0), (-2, 0)],
                Rot::Ccw => [(0, 1), (0, 1), (0, 1), (0, -1), (0, 2)],
            },
            Tetromino::O => {
                [match rot {
                    Rot::Init => (0, 0),
                    Rot::Cw => (0, -1),
                    Rot::Double => (-1, -1),
                    Rot::Ccw => (-1, 0),
                }; 5]
            }
        }
        .map(|(dx, dy)| Offset { dx, dy })
    }

    /// Returns relative coordinate for the blocks of pieces, taken from [How
    /// Guideline SRS _Really_ Works][guideline-srs].
    ///
    /// [guideline-srs]:
    ///     https://tetris.wiki/Super_Rotation_System#How_Guideline_SRS_Really_Works
    pub fn coordinates(self) -> [Offset; 4] {
        match self {
            Tetromino::I => [(-1, 0), (0, 0), (1, 0), (2, 0)],
            Tetromino::J => [(-1, 1), (-1, 0), (0, 0), (1, 0)],
            Tetromino::L => [(-1, 0), (0, 0), (1, 0), (1, 1)],
            Tetromino::O => [(0, 0), (1, 0), (1, 1), (0, 1)],
            Tetromino::S => [(-1, 0), (0, 0), (0, 1), (1, 1)],
            Tetromino::T => [(-1, 0), (0, 0), (0, 1), (1, 0)],
            Tetromino::Z => [(-1, 1), (0, 1), (0, 0), (1, 0)],
        }
        .map(|(dx, dy)| Offset { dx, dy })
    }

    /// Returns coordinates of a piece at a position.
    pub fn coordinates_at(self, rot: Rot, pos: Pos) -> [Pos; 4] {
        self.coordinates().map(|delta| pos + rot.apply(delta))
    }

    /// Returns the list of translations to try when rotating blocks. The first
    /// translation that works is selected.
    ///
    /// If no translations work, then the rotation fails.
    pub fn kick_translations(self, initial_rotation: Rot, final_rotation: Rot) -> [Offset; 5] {
        let initial_offsets = self.offsets(initial_rotation);
        let final_offsets = self.offsets(final_rotation);
        [0, 1, 2, 3, 4].map(|i| initial_offsets[i] - final_offsets[i])
    }
}
