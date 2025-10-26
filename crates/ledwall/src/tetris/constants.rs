use super::{Tetromino, Transform};

pub mod coordinates {
    use super::Transform;

    pub const PLAYFIELD_3X: Transform = Transform::huge([1, 2], [10, 20]);

    pub const PLAYFIELD: Transform = Transform::big([1, 23], [10, 20]);

    pub const HOLD_ON_TOP: bool = false;
    pub const HOLD_Y: i32 = if HOLD_ON_TOP { 23 } else { 50 };
    pub const NEXT_PIECE_Y: i32 = if HOLD_ON_TOP { 35 } else { 23 };

    pub const NEXT_PIECE: Transform = Transform::big([23, NEXT_PIECE_Y], [4, 4]);
    pub const NEXT_PIECE_2: Transform = Transform::small([25, NEXT_PIECE_Y + 9], [4, 4]);
    pub const NEXT_PIECE_3: Transform = Transform::small([25, NEXT_PIECE_Y + 14], [4, 4]);
    pub const NEXT_PIECE_4: Transform = Transform::small([25, NEXT_PIECE_Y + 19], [4, 4]);
    pub const HELD_PIECE: Transform = Transform::big([23, HOLD_Y], [4, 4]);
}

pub mod colors {
    use super::Tetromino;
    use crate::color::*;

    pub const DARKEN_STATIC_BLOCKS: f32 = 0.2;
    pub const DARKEN_GHOST: f32 = 0.6;
    pub const DARKEN_USED_HELD_PIECE: f32 = 0.5;

    pub const BACKGROUND: Rgb = BLACK;
    pub const PLAYFIELD_BORDER: Rgb = Rgb::from_hex(0x333333);
    pub const NEXT_PIECE_BORDER: Rgb = Rgb::from_hex(0x112222);
    pub const HELD_PIECE_BORDER: Rgb = Rgb::from_hex(0x112211);

    pub const fn piece(piece: Tetromino) -> Rgb {
        match piece {
            Tetromino::I => Rgb::from_hex(0x00FFFF),
            Tetromino::J => Rgb::from_hex(0x0033FF),
            Tetromino::L => Rgb::from_hex(0xFF6600),
            Tetromino::O => Rgb::from_hex(0xFFFF00),
            Tetromino::S => Rgb::from_hex(0x00FF00),
            Tetromino::T => Rgb::from_hex(0xFF00FF),
            Tetromino::Z => Rgb::from_hex(0xFF1111),
        }
    }
}

pub mod animations {
    pub mod hard_drop {
        pub const DURATION: f32 = 0.5; // seconds
        pub const TRAIL_LEN: f32 = 10.0; // blocks
        pub const TRAIL_OPACITY: f32 = 0.5;
    }

    pub mod lock {
        // must be shorter than `clear`
        pub const DURATION: f32 = 0.5; // seconds
    }

    pub mod clear {
        pub const SWIPE_DURATION: f32 = 0.25; //seconds
        pub const FADE_DURATION: f32 = 0.5; // seconds
        pub const DURATION: f32 = SWIPE_DURATION + FADE_DURATION;
    }
}
