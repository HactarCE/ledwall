use std::time::{Duration, Instant};

use ledwall::{App, FPS, HEIGHT, Input, WIDTH};
use macroquad::prelude::*;

const SCALE_FACTOR: f32 = 10.0;
const PADDING: f32 = 25.0;

const X_PADDING: f32 = PADDING;
const TOP_PADDING: f32 = PADDING;
const BOTTOM_PADDING: f32 = 30.0 + PADDING;

#[macroquad::main("ledwall")]
async fn main() {
    request_new_screen_size(
        32.0 * SCALE_FACTOR + X_PADDING * 2.0,
        64.0 * SCALE_FACTOR + TOP_PADDING + BOTTOM_PADDING,
    );

    let mut next_frame_time = Instant::now();
    let mut app = App::default();

    let mut rgba_buffer = vec![];

    rgb_to_rgba(&mut rgba_buffer, app.buffer());
    let texture = Texture2D::from_rgba8(WIDTH as u16, HEIGHT as u16, &rgba_buffer);
    texture.set_filter(FilterMode::Nearest);

    let mut show_fps = false;

    loop {
        // Wait for next frame
        let now = Instant::now();
        std::thread::sleep(next_frame_time.saturating_duration_since(now));
        next_frame_time = now + Duration::from_secs_f64(1.0 / FPS as f64);

        // Toggle FPS counter
        if is_key_pressed(KeyCode::F) {
            show_fps ^= true;
        }

        // Take input
        let input = Input {
            up: is_key_down(KeyCode::Up) || is_key_down(KeyCode::W),
            down: is_key_down(KeyCode::Down) || is_key_down(KeyCode::S),
            left: is_key_down(KeyCode::Left) || is_key_down(KeyCode::A),
            right: is_key_down(KeyCode::Right) || is_key_down(KeyCode::D),
            a: is_key_down(KeyCode::L),
            b: is_key_down(KeyCode::Comma),
            x: is_key_down(KeyCode::K),
            y: is_key_down(KeyCode::M),
            ..Default::default()
        };

        // Update app
        app.update(input);

        // Update display
        rgb_to_rgba(&mut rgba_buffer, app.buffer());
        texture.update_from_bytes(WIDTH as u32, HEIGHT as u32, &rgba_buffer);
        draw_texture_ex(
            &texture,
            X_PADDING,
            TOP_PADDING,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    WIDTH as f32 * SCALE_FACTOR,
                    HEIGHT as f32 * SCALE_FACTOR,
                )),
                ..Default::default()
            },
        );

        if show_fps {
            draw_fps();
        }

        next_frame().await;
    }
}

fn rgb_to_rgba(rgba_buffer: &mut Vec<u8>, rgb: &[[[u8; 3]; WIDTH]; HEIGHT]) {
    rgba_buffer.resize(WIDTH * HEIGHT * 4, 255);
    for (i, rgb) in rgb.as_flattened().iter().enumerate() {
        rgba_buffer[i * 4..i * 4 + 3].copy_from_slice(rgb);
    }
}
