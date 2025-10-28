use rpi_led_panel::*;

use crate::{FPS, HEIGHT, Rgb, Shell, WIDTH};

pub fn main() {
    let (mut matrix, mut canvas) = init_matrix(crate::DEFAULT_BRIGHTNESS);

    let mut shell = Shell::default();

    loop {
        // Take input
        let input = shell.read_gilrs_input();

        // Update state
        let output = shell.update(input);
        if let Some(new_brightness) = output.new_brightness {
            (matrix, canvas) = init_matrix(new_brightness);
        }

        // Update canvas
        for (y, row) in shell.frame_buffer().iter().enumerate() {
            for (x, &Rgb([r, g, b])) in row.iter().enumerate() {
                canvas.set_pixel(HEIGHT - 1 - y, x, r, g, b);
            }
        }

        // Update display and wait for next frame
        canvas = matrix.update_on_vsync(canvas);
    }
}

fn init_matrix(brightness: u8) -> (RGBMatrix, Box<Canvas>) {
    let config = RGBMatrixConfig {
        led_brightness: brightness * 5, // 0..=100
        hardware_mapping: HardwareMapping::adafruit_hat_pwm(),
        cols: HEIGHT,
        rows: WIDTH,
        refresh_rate: FPS,
        ..Default::default()
    };
    RGBMatrix::new(config, 0).expect("error initializing matrix")
}
