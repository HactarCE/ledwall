use std::time::Duration;

use ledwall_os::{App, FPS, HEIGHT, Input, WIDTH};
use rpi_led_panel::*;

// 0..=100
const BRIGHTNESS: u8 = 100;

fn main() {
    let mut config = RGBMatrixConfig::default();
    config.led_brightness = BRIGHTNESS;
    config.hardware_mapping = HardwareMapping::adafruit_hat_pwm();
    config.cols = ledwall_os::HEIGHT;
    config.rows = ledwall_os::WIDTH;
    let (mut matrix, mut canvas) = RGBMatrix::new(config, 0).expect("error initializing matrix");

    let mut app = App::default();

    if let Some(arg) = std::env::args().nth(1) {
        app.set_image(Some(std::fs::read(arg).unwrap()));
    }

    loop {
        // Take input
        let input = app.read_gilrs_input();

        // Update app
        app.update(input);

        // Update canvas
        for (y, row) in app.buffer().iter().enumerate() {
            for (x, &[r, g, b]) in row.iter().enumerate() {
                canvas.set_pixel(HEIGHT - 1 - y, x, r, g, b);
            }
        }

        // Update display and wait for next frame
        canvas = matrix.update_on_vsync(canvas);
    }
}
