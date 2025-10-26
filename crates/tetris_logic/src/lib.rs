//! Tetris game logic.
//!
//! Although the [Tetris wiki](https://tetris.wiki/) uses 1-indexed coordinates,
//! this crate uses 0-indexed coordinates.

use rand::RngCore;

mod config;
mod error;
mod input;
mod output;
mod piece;
mod playfield;
mod pos;
mod queue;
mod rotation;
mod time;

pub use config::{Config, Das, LockDown};
pub use error::{Blocked, Error, GameOver, HoldUsed};
pub use input::{FrameInput, InputState};
pub use output::FrameOutput;
pub use piece::Tetromino;
pub use playfield::Playfield;
pub use pos::{Offset, Pos};
pub use queue::Queue;
pub use rotation::Rot;
pub use time::GameTime;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(feature = "web-time")]
pub type DefaultTime = web_time::Instant;
#[cfg(not(feature = "web-time"))]
pub type DefaultTime = std::time::Instant;

/// Tetris game state.
pub struct Game<Time: GameTime = DefaultTime> {
    config: Config<Time>,
    playfield: Playfield,
    queue: Queue,

    input_state: InputState<Time>,

    /// Time of last frame.
    frame: Time,

    falling_piece: FallingPiece<Time>,

    held_piece: Option<Tetromino>,
    /// Whether the falling piece has already been held.
    hold_used: bool,

    rows_to_clear: Vec<i8>,

    /// Whether the game has ended.
    game_over: bool,
}

impl<Time: GameTime> Game<Time> {
    pub fn new(config: Config<Time>, first_frame: Time, rng: Box<dyn RngCore>) -> Self {
        let mut ret = Self {
            config,
            playfield: Playfield::new(config.width, config.height + config.buffer_height),
            queue: Queue::new(rng),

            input_state: InputState::default(),

            frame: first_frame,

            // dummy value; will be replaced
            falling_piece: FallingPiece {
                piece: Tetromino::I,
                rot: Rot::Init,
                pos: Pos::default(),
                frame_of_last_move: first_frame,
            },

            held_piece: None,
            hold_used: false,

            rows_to_clear: vec![0],

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

    fn set_falling_piece(&mut self, new_falling_piece: FallingPiece<Time>) -> Result<(), Blocked> {
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

    pub fn falling_piece(&self) -> FallingPiece<Time> {
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
    pub fn config(&self) -> &Config<Time> {
        &self.config
    }
    pub fn queue(&mut self) -> &mut Queue {
        &mut self.queue
    }
    pub fn held_piece(&self) -> Option<Tetromino> {
        self.held_piece
    }

    /// Advances the game to the next frame.
    pub fn step(
        &mut self,
        delta: Time::Duration,
        keys_down: FrameInput,
    ) -> Result<FrameOutput<Time>, GameOver> {
        if self.game_over {
            return Err(GameOver);
        }

        for row in self.rows_to_clear.drain(..).rev() {
            self.playfield.delete_row(row);
        }

        self.frame += delta;

        // Update input state.
        let actions_requested = self
            .input_state
            .update(self.config.das, self.frame, keys_down);

        let mut locked_piece = None;

        // // Move piece down automatically.
        // self.config.master_mode

        self.rows_to_clear = self.playfield.full_rows().collect();

        // Attempt actions.
        let actions_completed = FrameOutput {
            left: actions_requested.left.then(|| self.move_left()),
            right: actions_requested.right.then(|| self.move_right()),
            soft_drop: actions_requested.soft_drop.then(|| self.soft_drop()),
            hard_drop: actions_requested.hard_drop.then(|| {
                self.hard_drop().map(|[old_falling, locked]| {
                    locked_piece = Some(locked);
                    old_falling
                })
            }),
            rot_cw: actions_requested.rot_cw.then(|| self.rotate_cw()),
            rot_ccw: actions_requested.rot_ccw.then(|| self.rotate_ccw()),
            rot_180: actions_requested.rot_180.then(|| self.rotate_180()),
            hold: actions_requested.hold.then(|| self.hold()),
            locked_piece: None,
            rows_cleared: Some(self.rows_to_clear.clone()).filter(|list| !list.is_empty()),
        };

        // Check for game-over.
        if self.ghost_piece_pos().is_none() {
            self.game_over = true;
        }

        Ok(actions_completed)
    }
}

/// Inputs
impl<Time: GameTime> Game<Time> {
    /// Attempts to move the falling piece to the left.
    pub fn move_left(&mut self) -> Result<(), Blocked> {
        self.input_move(Offset::LEFT)
    }
    /// Attempts to move the falling piece to the right.
    pub fn move_right(&mut self) -> Result<(), Blocked> {
        self.input_move(Offset::RIGHT)
    }
    /// Attempts to soft-drop the falling piece.
    pub fn soft_drop(&mut self) -> Result<(), Blocked> {
        self.input_move(Offset::DOWN)
    }
    /// Hard-drops the falling piece.
    ///
    /// If successful, returns the old location of the falling piece and the
    /// location of the locked piece.
    pub fn hard_drop(&mut self) -> Result<[FallingPiece<Time>; 2], Blocked> {
        let old_falling_piece = self.falling_piece;
        if let Some(pos) = self.ghost_piece_pos() {
            self.falling_piece.pos = pos;
        }
        let locked_piece = self.falling_piece;
        self.lock_falling_piece()?;
        Ok([old_falling_piece, locked_piece])
    }
    /// Attempts to rotate the piece 90° clockwise.
    ///
    /// If successful, returns the old location of the falling piece.
    pub fn rotate_cw(&mut self) -> Result<(), Blocked> {
        self.input_rotate(Rot::rot_cw)
    }
    /// Attempts to rotate the piece 180°.
    ///
    /// If successful, returns the old location of the falling piece.
    pub fn rotate_180(&mut self) -> Result<(), Blocked> {
        self.input_rotate(Rot::rot_180)
    }
    /// Attempts to rotate the piece 90° counterclockwise.
    ///
    /// If successful, returns the old location of the falling piece.
    pub fn rotate_ccw(&mut self) -> Result<(), Blocked> {
        self.input_rotate(Rot::rot_ccw)
    }
    /// Attempts to swap the falling piece with the held piece.
    ///
    /// This fails only if hold was used on the last piece.
    pub fn hold(&mut self) -> Result<(), HoldUsed> {
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
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FallingPiece<Time: GameTime = DefaultTime> {
    pub piece: Tetromino,
    pub rot: Rot,
    pub pos: Pos,
    pub frame_of_last_move: Time,
}
