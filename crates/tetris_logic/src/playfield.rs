use crate::{Blocked, Pos, Rot, Tetromino};

/// Grid in which tetrominos fall.
pub struct Playfield {
    width: u8,
    height: u8,
    blocks: Box<[Option<Tetromino>]>,
}

impl Playfield {
    pub fn new(width: u8, height: u8) -> Self {
        assert!(width <= i8::MAX as u8);
        assert!(height <= i8::MAX as u8);
        Self {
            width,
            height,
            blocks: vec![None; width as usize * height as usize].into_boxed_slice(),
        }
    }

    fn pos_to_index(&self, pos: Pos) -> Option<usize> {
        ((0..self.width as i8).contains(&pos.x) && (0..self.height as i8).contains(&pos.y))
            .then(|| pos.y as usize * self.width as usize + pos.x as usize)
    }

    pub fn get(&self, pos: Pos) -> Option<Option<Tetromino>> {
        Some(self.blocks[self.pos_to_index(pos)?])
    }
    pub fn get_mut(&mut self, pos: Pos) -> Option<&mut Option<Tetromino>> {
        Some(&mut self.blocks[self.pos_to_index(pos)?])
    }
    #[track_caller]
    pub fn set(&mut self, pos: Pos, block: Option<Tetromino>) {
        *self.get_mut(pos).expect("pos out of bounds") = block;
    }

    pub fn can_place_piece(&self, piece: Tetromino, rot: Rot, pos: Pos) -> bool {
        piece
            .coordinates_at(rot, pos)
            .into_iter()
            .all(|pos| self.get(pos) == Some(None))
    }

    pub fn place_piece(&mut self, piece: Tetromino, rot: Rot, pos: Pos) -> Result<(), Blocked> {
        if !self.can_place_piece(piece, rot, pos) {
            return Err(Blocked);
        }
        for block_pos in piece.coordinates_at(rot, pos) {
            self.set(block_pos, Some(piece));
        }
        Ok(())
    }

    pub fn width(&self) -> u8 {
        self.width
    }
    pub fn height(&self) -> u8 {
        self.height
    }
}
