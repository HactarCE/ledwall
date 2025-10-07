use rpi_led_matrix::*;

// 0..=100
const BRIGHTNESS: usize = 10;

fn main() {
    let mut matrix_options = LedMatrixOptions::new();
    matrix_options.set_brightness(BRIGHTNESS);
    matrix_options.set_hardware_mapping("adafruit-hat");
    matrix_options.set_limit_refresh(180);
    matrix_options.set_cols(ledwall::WIDTH as u32);
    matrix_options.set_rows(ledwall::HEIGHT as u32);

    let runtime_options = LedRuntimeOptions::new();

    let matrix = LedMatrix::new(Some(matrix_options), Some(runtime_options)).unwrap();
    let mut canvas = matrix.offscreen_canvas();
    for red in (0..255).step_by(16) {
        for green in (0..255).step_by(16) {
            for blue in (0..255).step_by(16) {
                canvas.fill(&LedColor { red, green, blue });
                canvas = matrix.swap(canvas);
            }
        }
    }
}
