use rpi_led_panel::*;

use crate::{FPS, HEIGHT, Rgb, Shell, WIDTH};

pub fn main() {
    let (mut matrix, mut canvas) = init_matrix(crate::DEFAULT_BRIGHTNESS);

    let mut shell = Shell::default();

    loop {
        // Take input
        let (blue, green) = shell.read_gilrs_input();

        // Update state
        let output = shell.update(blue, green);
        if let Some(new_brightness) = output.new_brightness_percent {
            matrix.set_brightness(new_brightness);
        }

        // Update canvas
        for (y, row) in shell.frame_buffer().iter().enumerate() {
            for (x, &Rgb([r, g, b])) in row.iter().enumerate() {
                canvas.set_pixel(HEIGHT - 1 - y, x, r, g, b);
            }
        }

        // Update display and wait for next frame
        canvas = matrix
            .update_on_vsync(canvas)
            .expect("error updating matrix");
    }
}

fn init_matrix(brightness: u8) -> (RGBMatrix, Canvas) {
    let config = RGBMatrixConfig {
        led_brightness: brightness * 5, // 0..=100
        hardware_mapping: HardwareMapping::adafruit_hat_pwm(),
        cols: HEIGHT,
        rows: WIDTH,
        refresh_rate: FPS,
        dither_bits: 2,
        drop_priv_user: String::from(env!("USER")),
        ..Default::default()
    };
    RGBMatrix::new(config).expect("error initializing matrix")
}
