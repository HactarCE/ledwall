/// Hold already used.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct HoldUsed;

/// All allowed piece positions are blocked.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Blocked;

/// Error condition for an operation.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Error {
    /// No piece is currently falling.
    NoFallingPiece,
    /// Hold already used.
    HoldUsed,
    /// All allowed piece positions are blocked.
    Blocked,
}
