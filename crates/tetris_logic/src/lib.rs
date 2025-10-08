//! Tetris game logic.
//!
//! Although the [Tetris wiki](https://tetris.wiki/) uses 1-indexed coordinates,
//! this crate uses 0-indexed coordinates.

use rand::RngCore;

mod config;
mod error;
mod piece;
mod playfield;
mod pos;
mod queue;
mod rotation;

pub use config::Config;
pub use error::{Blocked, Error, HoldUsed};
pub use piece::Tetromino;
pub use playfield::Playfield;
pub use pos::{Offset, Pos};
pub use queue::Queue;
pub use rotation::Rot;

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub struct Game {
    config: Config,
    playfield: Playfield,
    queue: Queue,

    /// Current frame index at 30 FPS.
    frame: u64,

    falling_piece: FallingPiece,

    held_piece: Option<Tetromino>,
    /// Whether the falling piece has already been held.
    hold_used: bool,

    /// Whether the game has ended.
    game_over: bool,
}

impl Game {
    pub fn new(config: Config, rng: Box<dyn RngCore>) -> Self {
        let mut ret = Self {
            config,
            playfield: Playfield::new(config.width, config.height + config.buffer_height),
            queue: Queue::new(rng),

            frame: 0,

            // dummy value; will be replaced
            falling_piece: FallingPiece {
                piece: Tetromino::I,
                rot: Rot::Init,
                pos: Pos::default(),
                frame_of_last_move: 0,
            },

            held_piece: None,
            hold_used: false,

            game_over: false,
        };
        ret.next_piece();
        ret
    }

    fn next_piece(&mut self) {
        let new_piece = self.queue.pop_piece();
        self.init_falling_piece(new_piece);
    }

    fn init_falling_piece(&mut self, piece: Tetromino) {
        self.falling_piece = FallingPiece {
            piece,
            rot: Rot::Init,
            pos: self.config.spawn_pos,
            frame_of_last_move: self.frame,
        };
        if self.ghost_piece_pos().is_none() {
            self.game_over = true; // piece was spawned overlapping a block
        }
    }

    fn set_falling_piece(&mut self, new_falling_piece: FallingPiece) -> Result<(), Blocked> {
        if self.playfield.can_place_piece(
            new_falling_piece.piece,
            new_falling_piece.rot,
            new_falling_piece.pos,
        ) {
            self.falling_piece = new_falling_piece;
            Ok(())
        } else {
            Err(Blocked)
        }
    }

    fn lock_falling_piece(&mut self) -> Result<(), Blocked> {
        self.playfield.place_piece(
            self.falling_piece.piece,
            self.falling_piece.rot,
            self.falling_piece.pos,
        )?;
        self.next_piece();
        self.hold_used = false;
        Ok(())
    }

    pub fn falling_piece(&self) -> FallingPiece {
        self.falling_piece
    }

    pub fn ghost_piece_pos(&self) -> Option<Pos> {
        let falling_piece = self.falling_piece;
        let x = falling_piece.pos.x;
        (0..=falling_piece.pos.y)
            .rev()
            .map(|y| Pos { x, y })
            .take_while(|&pos| {
                self.playfield
                    .can_place_piece(falling_piece.piece, falling_piece.rot, pos)
            })
            .last()
    }

    pub fn playfield(&self) -> &Playfield {
        &self.playfield
    }
    pub fn config(&self) -> &Config {
        &self.config
    }
    pub fn queue(&mut self) -> &mut Queue {
        &mut self.queue
    }
    pub fn held_piece(&self) -> Option<Tetromino> {
        self.held_piece
    }
}

/// Input
impl Game {
    /// Attempts to move the falling piece to the left.
    pub fn input_move_left(&mut self) -> Result<(), Blocked> {
        self.input_move(Offset { dx: -1, dy: 0 })
    }
    /// Attempts to move the falling piece to the right.
    pub fn input_move_right(&mut self) -> Result<(), Blocked> {
        self.input_move(Offset { dx: 1, dy: 0 })
    }
    /// Soft-drops the falling piece and returns whether it moved.
    pub fn input_soft_drop(&mut self) -> Result<(), Blocked> {
        self.input_move(Offset { dx: 0, dy: -1 })
    }
    /// Hard-drops the falling piece.
    pub fn input_hard_drop(&mut self) -> Result<(), Blocked> {
        if let Some(pos) = self.ghost_piece_pos() {
            self.falling_piece.pos = pos;
        }
        self.lock_falling_piece()
    }
    /// Attempts to rotate the piece 90° clockwise.
    pub fn input_rotate_cw(&mut self) -> Result<(), Blocked> {
        self.input_rotate(Rot::rot_cw)
    }
    /// Attempts to rotate the piece 180°.
    pub fn input_rotate_180(&mut self) -> Result<(), Blocked> {
        self.input_rotate(Rot::rot_180)
    }
    /// Attempts to rotate the piece 90° counterclockwise.
    pub fn input_rotate_ccw(&mut self) -> Result<(), Blocked> {
        self.input_rotate(Rot::rot_ccw)
    }
    /// Swaps the falling piece with the held piece and returns whether it was
    /// successful.
    ///
    /// This fails only if hold was used on the last piece.
    pub fn input_hold(&mut self) -> Result<(), HoldUsed> {
        if self.hold_used {
            return Err(HoldUsed);
        }
        self.hold_used = true;
        let new_falling_piece = self
            .held_piece
            .replace(self.falling_piece.piece)
            .unwrap_or_else(|| self.queue.pop_piece());
        self.init_falling_piece(new_falling_piece);
        Ok(())
    }

    fn input_move(&mut self, delta: Offset) -> Result<(), Blocked> {
        let mut new_falling_piece = self.falling_piece;
        new_falling_piece.pos += delta;
        new_falling_piece.frame_of_last_move = self.frame;
        self.set_falling_piece(new_falling_piece)
    }

    fn input_rotate(&mut self, apply_rotation: fn(Rot) -> Rot) -> Result<(), Blocked> {
        let initial_rotation = self.falling_piece.rot;
        let final_rotation = apply_rotation(initial_rotation);
        for kick in self
            .falling_piece
            .piece
            .kick_translations(initial_rotation, final_rotation)
        {
            let result = self.set_falling_piece(FallingPiece {
                piece: self.falling_piece.piece,
                rot: final_rotation,
                pos: self.falling_piece.pos + kick,
                frame_of_last_move: self.frame,
            });
            if result.is_ok() {
                return Ok(());
            }
        }
        Err(Blocked)
    }

    pub fn next_frame(&mut self) {
        self.frame += 1;

        // Check for
        if self.ghost_piece_pos().is_none() {
            self.game_over = true;
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FallingPiece {
    pub piece: Tetromino,
    pub rot: Rot,
    pub pos: Pos,
    pub frame_of_last_move: u64,
}
