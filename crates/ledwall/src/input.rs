use std::time::{Duration, Instant};

const DELAY: f32 = 1.0 / 3.0; // number of seconds
const RATE: f32 = 1.0 / 16.0; // number of seconds

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Input {
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

impl Input {
    pub fn newly_pressed_compared_to(self, last_frame_input: Input) -> Input {
        Self {
            up: self.up && !last_frame_input.up,
            down: self.down && !last_frame_input.down,
            left: self.left && !last_frame_input.left,
            right: self.right && !last_frame_input.right,
            a: self.a && !last_frame_input.a,
            b: self.b && !last_frame_input.b,
            x: self.x && !last_frame_input.x,
            y: self.y && !last_frame_input.y,
            l: self.l && !last_frame_input.l,
            r: self.r && !last_frame_input.r,
            lt: self.lt && !last_frame_input.lt,
            rt: self.rt && !last_frame_input.rt,
            plus: self.plus && !last_frame_input.plus,
            minus: self.minus && !last_frame_input.minus,
            star: self.star && !last_frame_input.star,
            heart: self.heart && !last_frame_input.heart,
        }
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
