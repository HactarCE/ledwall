mod app;
mod input;
mod tetris;

pub use app::App;
pub use input::Input;

#[cfg_attr(feature = "rpi-led-panel", path = "frontend_led_panel.rs")]
#[cfg_attr(feature = "macroquad", path = "frontend_macroquad.rs")]
mod frontend;

pub type Rgb = [u8; 3];
pub type FrameBuffer = [[Rgb; WIDTH]; HEIGHT];

pub const BLACK: Rgb = [0_u8; 3];
pub const WHITE: Rgb = [255_u8; 3];

pub const FPS: usize = 60;
pub const WIDTH: usize = 32;
pub const HEIGHT: usize = 64;

fn main() {
    frontend::main();
}
