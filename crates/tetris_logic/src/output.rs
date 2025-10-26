use crate::{Blocked, FallingPiece, GameTime, HoldUsed};

/// Result of simulating a frame, including which actions succeeded on a frame.
///
/// This can be used, e.g., to play sound effects.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct FrameOutput<Time: GameTime> {
    /// User attempt to move the falling piece to the left (auto-repeats).
    pub left: Option<Result<(), Blocked>>,
    /// User attempt to move the falling piece to the right (auto-repeats).
    pub right: Option<Result<(), Blocked>>,
    /// User attempt to soft-drop the falling piece and (auto-repeats).
    pub soft_drop: Option<Result<(), Blocked>>,
    /// User hard-dropped the falling piece.
    ///
    /// Contains the old location of the falling piece.
    pub hard_drop: Option<Result<FallingPiece<Time>, Blocked>>,
    /// User attempt to rotate the piece 90° clockwise.
    pub rot_cw: Option<Result<(), Blocked>>,
    /// User attempt to rotate the piece 90° counterclockwise.
    pub rot_ccw: Option<Result<(), Blocked>>,
    /// User attempt to rotate the piece 180°.
    pub rot_180: Option<Result<(), Blocked>>,
    /// User attempt to swaps the falling piece with the held piece.
    pub hold: Option<Result<(), HoldUsed>>,

    /// Piece that was just locked into place.
    pub locked_piece: Option<FallingPiece<Time>>,

    /// Y coordinates of the rows cleared on this frame.
    ///
    /// To display an animation, stop calling `step()` while displaying the
    /// animation.
    pub rows_cleared: Option<Vec<i8>>,
}
