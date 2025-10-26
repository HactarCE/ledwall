mod app;
mod color;
mod input;
mod tetris;

pub use app::App;
pub use color::*;
pub use input::Input;

#[cfg_attr(feature = "rpi-led-panel", path = "frontend_led_panel.rs")]
#[cfg_attr(feature = "macroquad", path = "frontend_macroquad.rs")]
mod frontend;

pub type FrameBuffer = [[Rgb; WIDTH]; HEIGHT];

pub const FPS: usize = 60;
pub const WIDTH: usize = 32;
pub const HEIGHT: usize = 64;

fn main() {
    frontend::main();
}

pub fn mix<T>(a: T, b: T, t: f32) -> T
where
    T: std::ops::Mul<f32, Output = T>,
    T: std::ops::Add<Output = T>,
{
    a * (1.0 - t) + b * t
}

pub fn xy_is_in_frame([x, y]: [usize; 2]) -> bool {
    (0..crate::WIDTH).contains(&x) && (0..crate::HEIGHT).contains(&y)
}
