use std::fmt;
use std::ops::{Index, IndexMut};

use rand::distr::{Distribution, StandardUniform};
use rand::{Rng, seq::IndexedRandom};

use crate::{Axis, Pos4, Sign};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Facet {
    pub axis: Axis,
    pub sign: Sign,
}

impl Facet {
    pub const R: Self = Self::from_id(0);
    pub const L: Self = Self::from_id(1);
    pub const U: Self = Self::from_id(2);
    pub const D: Self = Self::from_id(3);
    pub const F: Self = Self::from_id(4);
    pub const B: Self = Self::from_id(5);
    pub const O: Self = Self::from_id(6);
    pub const I: Self = Self::from_id(7);
    pub const ALL: [Self; 8] = [
        Self::R,
        Self::L,
        Self::U,
        Self::D,
        Self::F,
        Self::B,
        Self::O,
        Self::I,
    ];

    pub fn id(self) -> u8 {
        (self.axis.id() << 1) | self.sign as u8
    }

    pub const fn from_id(id: u8) -> Self {
        Self {
            axis: Axis::from_id(id >> 1),
            sign: if id & 1 == 0 { Sign::Pos } else { Sign::Neg },
        }
    }

    pub fn pieces(self) -> impl Iterator<Item = Piece> {
        self.stickers().map(|sticker| sticker.piece())
    }

    pub fn stickers(self) -> impl Iterator<Item = Sticker> {
        [-1, 0, 1].into_iter().flat_map(move |q| {
            [-1, 0, 1].into_iter().flat_map(move |r| {
                [-1, 0, 1].into_iter().map(move |s| {
                    let mut pos = Pos4([q, r, s, self.sign.to_i8() * 2]);
                    pos.0.swap(self.axis as usize, 3);
                    Sticker(pos)
                })
            })
        })
    }

    pub fn center_piece(self) -> Piece {
        self.center_sticker().piece()
    }

    pub fn center_sticker(self) -> Sticker {
        let mut ret = Pos4::default();
        ret[self.axis] = self.sign * 2;
        Sticker(ret)
    }

    pub fn pos(axis: Axis) -> Self {
        let sign = Sign::Pos;
        Self { axis, sign }
    }
    pub fn neg(axis: Axis) -> Self {
        let sign = Sign::Neg;
        Self { axis, sign }
    }

    pub fn has_pos(self, pos: Pos4) -> bool {
        Sign::try_from_i8(pos[self.axis]) == Some(self.sign)
    }
    pub fn has_piece(self, piece: Piece) -> bool {
        self.has_pos(piece.pos())
    }
    pub fn has_sticker(self, sticker: Sticker) -> bool {
        self.has_pos(sticker.pos())
    }
}

impl fmt::Display for Facet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &"RLUDFBOI"[self.id() as usize..][..1])
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Turn {
    pub facet: Option<Facet>,
    pub from: Facet,
    pub to: Facet,
}

impl fmt::Display for Turn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.facet {
            Some(facet) => write!(f, "{facet}")?,
            None => write!(f, "@")?,
        }
        write!(f, "[{}->{}]", self.from, self.to)
    }
}

impl Distribution<Turn> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Turn {
        let [ax1, ax2, ax3] = Axis::ALL.choose_multiple_array(rng).unwrap();
        Turn {
            facet: Some(Facet {
                axis: ax1,
                sign: *[Sign::Pos, Sign::Neg].choose(rng).unwrap(),
            }),
            from: Facet::pos(ax2),
            to: Facet::pos(ax3),
        }
    }
}

impl Turn {
    pub fn inverse(self) -> Self {
        Turn {
            facet: self.facet,
            from: self.to,
            to: self.from,
        }
    }
}

/// Position of a piece on the puzzle.
///
/// All of these coordinates must be ±1.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Piece(Pos4);

impl Piece {
    pub const CORE: Self = Self(Pos4([0; 4]));

    pub fn iter_all() -> impl Iterator<Item = Piece> {
        [-1, 0, 1].into_iter().flat_map(move |w| {
            [-1, 0, 1].into_iter().flat_map(move |z| {
                [-1, 0, 1].into_iter().flat_map(move |y| {
                    [-1, 0, 1].into_iter().map(move |x| {
                        let coordinates = [x, y, z, w];
                        Piece(Pos4(coordinates))
                    })
                })
            })
        })
    }

    pub fn stickers(self) -> impl Iterator<Item = Sticker> {
        Axis::ALL
            .into_iter()
            .filter(move |&axis| self.0[axis] != 0)
            .map(move |axis| {
                let mut pos = self.0;
                pos[axis] *= 2;
                Sticker(pos)
            })
    }

    pub fn pos(self) -> Pos4 {
        self.0
    }

    /// Rotates a piece from `from` to `to`.
    ///
    /// Returns `None` if `from` and `to` are on the same axis.
    #[must_use]
    pub fn rot(self, from: Facet, to: Facet) -> Option<Self> {
        self.0.rot(from, to).map(Self)
    }
}

/// Position of a sticker on the puzzle.
///
/// Exactly one of these coordinates must be ±2. All others must be ±1.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Sticker(Pos4);

impl Sticker {
    pub fn piece(self) -> Piece {
        Piece(Pos4(self.0.0.map(|coord| coord.clamp(-1, 1))))
    }

    pub fn facet(self) -> Facet {
        let axis = match self.0.0 {
            [-2 | 2, _, _, _] => Axis::X,
            [_, -2 | 2, _, _] => Axis::Y,
            [_, _, -2 | 2, _] => Axis::Z,
            [_, _, _, -2 | 2] => Axis::W,
            _ => panic!("bad sticker pos"),
        };
        let sign = Sign::try_from_i8(self.0[axis]).unwrap();
        Facet { axis, sign }
    }

    fn index(self) -> usize {
        let mut coordinates = self.0.0;
        coordinates.swap(self.facet().axis as usize, 3);
        let [q, r, s, _] = coordinates;
        self.facet().id() as usize * 27
            + (q + 1) as usize
            + (r + 1) as usize * 3
            + (s + 1) as usize * 9
    }

    pub fn iter_all() -> impl Iterator<Item = Self> {
        Piece::iter_all().flat_map(Piece::stickers)
    }

    pub fn pos(self) -> Pos4 {
        self.0
    }

    /// Rotates a position from `from` to `to`.
    ///
    /// Returns `None` if `from` and `to` are on the same axis.
    #[must_use]
    pub fn rot(self, from: Facet, to: Facet) -> Option<Self> {
        self.0.rot(from, to).map(Self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Puzzle {
    /// Map from sticker index to color
    pub stickers: [Facet; 3 * 3 * 3 * 8],
}

impl Index<Sticker> for Puzzle {
    type Output = Facet;

    fn index(&self, sticker: Sticker) -> &Self::Output {
        &self.stickers[sticker.index()]
    }
}

impl IndexMut<Sticker> for Puzzle {
    fn index_mut(&mut self, sticker: Sticker) -> &mut Self::Output {
        &mut self.stickers[sticker.index()]
    }
}

impl Default for Puzzle {
    fn default() -> Self {
        Self::new_solved()
    }
}

impl Puzzle {
    pub fn new_solved() -> Self {
        let mut stickers = [Facet::R; 3 * 3 * 3 * 8];
        for facet in Facet::ALL {
            stickers[facet.id() as usize * 27..][..27].fill(facet);
        }

        Puzzle { stickers }
    }

    pub fn facets(&self) -> &[[Facet; 3 * 3 * 3]; 8] {
        self.stickers.as_chunks().0.try_into().unwrap()
    }

    pub fn is_solved(self) -> bool {
        self.facets()
            .iter()
            .all(|stickers| stickers[1..].iter().all(|&sticker| sticker == stickers[0]))
    }

    /// Does a turn on the puzzle and returns whether it succeeded.
    #[must_use]
    pub fn do_turn(&mut self, turn: Turn) -> bool {
        let Turn { facet, from, to } = turn;
        match facet {
            Some(f) => self.turn_facet(f, from, to),
            None => self.turn_whole_puzzle(from, to),
        }
        .is_ok()
    }

    fn turn_facet(&mut self, facet: Facet, from: Facet, to: Facet) -> Result<(), ()> {
        if facet.axis == from.axis || facet.axis == to.axis {
            return Err(());
        }

        let old_state = self.clone();
        for sticker in facet.pieces().flat_map(Piece::stickers) {
            self[sticker.rot(from, to).ok_or(())?] = old_state[sticker];
        }

        Ok(())
    }

    fn turn_whole_puzzle(&mut self, from: Facet, to: Facet) -> Result<(), ()> {
        let old_state = self.clone();
        for sticker in Sticker::iter_all() {
            self[sticker.rot(from, to).ok_or(())?] = old_state[sticker];
        }
        Ok(())
    }

    pub fn scramble(&mut self, rng: &mut impl Rng) {
        self.scramble_n(rng, crate::SCRAMBLE_MOVE_COUNT);
    }

    pub fn scramble_n(&mut self, rng: &mut impl Rng, move_count: usize) {
        for _ in 0..move_count {
            assert!(self.do_turn(rng.random()), "scramble move failed");
        }
    }
}
