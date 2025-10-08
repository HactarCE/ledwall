use crate::Pos;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Config {
    /// Width of the visible playfield.
    pub width: u8,
    /// Height of the visible playfield. Everywhere above this is the vanish
    /// zone.
    pub height: u8,
    /// Height above the visible playfield of the buffer zone, which exists
    /// within the infinite vanish zone.
    pub buffer_height: u8,

    /// Spawn position for blocks. Blocks move down immediately after appearing.
    pub spawn_pos: Pos,

    /// Lock down behavior.
    pub lock_down: LockDown,
    /// Whether to decrease the lock down delay value per level when the gravity
    /// is 20G.
    pub master_mode: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            width: 10,
            height: 20,
            buffer_height: 20,

            spawn_pos: Pos::new(4, 20),

            lock_down: LockDown::default(),
            master_mode: false,
        }
    }
}

/// Lock down behavior.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LockDown {
    /// Reset the lock down timer when the piece is moved or rotated.
    ///
    /// Also called **Infinite Placement Lock Down**.
    #[default]
    Infinity,
    // TODO: add `MoveReset` and `Classic`
    // /// Reset the lock down timer the first 15 times a piece is moved or
    // /// rotated.
    // ///
    // /// Also called **Extended Placement Lock Down**.
    // #[default]
    // MoveReset,
    // /// Reset the lock down timer when the piece moves down.
    // ///
    // /// This is also called "step reset."
    // Classic,
}
