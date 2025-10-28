#[macro_use]
mod image;
#[macro_use]
mod animation;

mod activities;
mod color;
mod frame_buffer;
mod input;
mod shell;
mod traits;
mod widgets;

use std::ops::Range;

pub use animation::{Animation, AnimationFrame, draw_opt_animation, step_opt_animation};
pub use color::*;
pub use frame_buffer::{FrameBuffer, FrameBufferRect};
pub use image::StaticImage;
pub use input::{Input, KeyRepeat};
pub use shell::Shell;
pub use traits::{Activity, Widget};

#[cfg_attr(feature = "rpi-led-panel", path = "frontend_led_panel.rs")]
#[cfg_attr(feature = "macroquad", path = "frontend_macroquad.rs")]
mod frontend;

pub const FPS: usize = 120;
pub const WIDTH: usize = 32;
pub const HEIGHT: usize = 64;

pub const DEFAULT_VOLUME: u8 = 10; // 0..=20
pub const DEFAULT_BRIGHTNESS: u8 = 12; // 0..=20

fn main() {
    frontend::main();
}

pub fn mix(range: Range<f32>, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    range.start * (1.0 - t) + range.end * t
}

pub fn map_range(parameter: f32, in_range: Range<f32>, out_range: Range<f32>) -> f32 {
    assert!(in_range.start <= in_range.end, "range is reversed");
    if parameter <= in_range.start {
        out_range.start
    } else if parameter >= in_range.end {
        out_range.end
    } else {
        let t = (parameter - in_range.start) / (in_range.end - in_range.start);
        mix(out_range, t)
    }
}

pub fn xy_is_in_frame([x, y]: [usize; 2]) -> bool {
    (0..crate::WIDTH).contains(&x) && (0..crate::HEIGHT).contains(&y)
}
