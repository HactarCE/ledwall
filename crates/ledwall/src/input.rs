use std::{
    ops::{BitAnd, BitOr, BitXor, Not},
    time::{Duration, Instant},
};

const DELAY: f32 = 1.0 / 3.0; // number of seconds
const RATE: f32 = 1.0 / 16.0; // number of seconds

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FullInput {
    /// Blue controller input state.
    pub blue: Option<ControllerInput>,
    /// Green controller input state.
    pub green: Option<ControllerInput>,
}

impl FullInput {
    /// Returns inputs on either controller.
    pub fn any(self) -> ControllerInput {
        let blue = self.blue.unwrap_or_default();
        let green = self.green.unwrap_or_default();
        ControllerInput {
            current: blue.current | green.current,
            previous: blue.previous | green.previous,
        }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ControllerInput {
    /// Buttons currently held down.
    pub current: Buttons,
    /// Buttons held down last frame.
    pub previous: Buttons,
}

impl ControllerInput {
    /// Returns the buttons that were pressed on the current frame.
    pub fn pressed(self) -> Buttons {
        self.current & !self.previous
    }
    /// Returns the buttons that were released on the current frame.
    pub fn released(self) -> Buttons {
        !self.current & self.previous
    }
}

/// Boolean for each controller button.
///
/// Depending on context, this may represent buttons currently held, buttons
/// newly pressed, or any other boolean property.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Buttons {
    // D pad
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,

    // Thumb buttons
    pub a: bool,
    pub b: bool,
    pub x: bool,
    pub y: bool,

    // Shoulder buttons
    pub l: bool,
    pub r: bool,
    pub lt: bool,
    pub rt: bool,

    // Middle buttons
    pub plus: bool,
    pub minus: bool,
    pub star: bool,
    pub heart: bool,
}

// tt muncher! because I'm feeling quirky
macro_rules! construct_fieldwise {
    (@[$field:ident] [$($done:tt)*] []) => {
        $($done)*
    };
    (@[$field:ident] [$($done:tt)*] [# $($next:tt)*]) => {
        construct_fieldwise!(@[$field] [$($done)* $field] [$($next)*])
    };
    (@[$field:ident] [$($done:tt)*] [$tok:tt $($next:tt)*]) => {
        construct_fieldwise!(@[$field] [$($done)* $tok] [$($next)*])
    };
    ($struct:ident; @[$($field:ident)+] $inner:tt) => {
        $struct {
            $( $field: construct_fieldwise!(@[$field] [] $inner), )+
        }
    };
    ($struct:ident, $($tok:tt)*) => {
        construct_fieldwise!(
            $struct;
            @[up down left right a b x y l r lt rt plus minus star heart]
            [$($tok)*]
        )
    };
}

impl BitOr for Buttons {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        construct_fieldwise!(Self, self.# | rhs.#)
    }
}

impl BitAnd for Buttons {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        construct_fieldwise!(Self, self.# & rhs.#)
    }
}

impl BitXor for Buttons {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        construct_fieldwise!(Self, self.# ^ rhs.#)
    }
}

impl Not for Buttons {
    type Output = Self;

    fn not(self) -> Self::Output {
        construct_fieldwise!(Self, !self.#)
    }
}

#[derive(Debug, Default, Clone)]
pub enum KeyRepeat {
    #[default]
    Released,
    Pressed(Instant),
    Repeating(Instant),
}

impl KeyRepeat {
    pub fn new() -> Self {
        Self::Released
    }

    pub fn update(&mut self, is_down: bool) -> bool {
        let now = Instant::now();
        match is_down {
            true => match self {
                Self::Released => {
                    *self = Self::Pressed(now + Duration::from_secs_f32(DELAY));
                    true
                }
                Self::Pressed(time_to_repeat) | Self::Repeating(time_to_repeat) => {
                    if now >= *time_to_repeat {
                        *self = Self::Repeating(*time_to_repeat + Duration::from_secs_f32(RATE));
                        true
                    } else {
                        false
                    }
                }
            },
            false => {
                *self = Self::Released;
                false
            }
        }
    }
}
