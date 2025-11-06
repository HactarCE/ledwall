use std::f32::consts::PI;

use tetris_logic::{FallingPiece, Pos};

use super::{Transform, colors, constants};
use crate::{Animation, AnimationFrame, FPS, FrameBufferRect, Rgb};

#[derive(Debug, Default)]
pub struct SoonToLockAnimation {
    frame: u32,
}
impl_animation_frame!(
    SoonToLockAnimation,
    constants::animations::soon_to_lock::DURATION
);
impl SoonToLockAnimation {
    pub fn reset(&mut self) {
        self.frame = 0;
    }

    pub fn step(&mut self) {
        self.frame += 1;
        if self.t() >= 1.0 {
            self.reset();
        }
    }

    pub fn modify_color(&self, color: Rgb) -> Rgb {
        let sin = (self.t() * PI).sin();
        // sin^2 smoothly varies from 0 to 1 and back
        color.lighten(sin * sin)
    }
}

/// Animation when a piece locks into place.
#[derive(Debug)]
pub struct LockedAnimation {
    frame: u32,
    locked_piece: FallingPiece<u64>,
}
impl_animation_frame!(LockedAnimation, constants::animations::locked::DURATION);
impl LockedAnimation {
    pub fn new(locked_piece: FallingPiece<u64>) -> Self {
        Self {
            frame: 0,
            locked_piece,
        }
    }
}
impl Animation<Transform> for LockedAnimation {
    fn draw(&self, fb: &mut FrameBufferRect<'_>, tf: Transform) {
        let t = self.t();

        let color = colors::piece(self.locked_piece.piece)
            .darken(colors::DARKEN_STATIC_BLOCKS)
            .lighten(1.0 - t);

        for pos in self.locked_piece.coordinates() {
            tf.fill_block(fb, pos, color);
        }
    }
}

/// Animation when hard-dropping a piece more than a few tiles.
#[derive(Debug)]
pub struct HardDropAnimation {
    frame: u32,
    trail_len: i8,
    end_piece: FallingPiece<u64>,
}
impl_animation_frame!(
    HardDropAnimation,
    constants::animations::hard_drop::DURATION
);
impl HardDropAnimation {
    pub fn new(trail_len: i8, end_piece: FallingPiece<u64>) -> Self {
        Self {
            frame: 0,
            trail_len,
            end_piece,
        }
    }
}
impl Animation<Transform> for HardDropAnimation {
    fn draw(&self, fb: &mut FrameBufferRect<'_>, tf: Transform) {
        use constants::animations::hard_drop::*;

        if self.trail_len == 0 {
            return;
        }

        let global_t = self.t();

        let color = colors::piece(self.end_piece.piece);

        let mut top_blocks = self.end_piece.coordinates().to_vec();
        top_blocks.sort_by_key(|&Pos { x, y }| (x, std::cmp::Reverse(y)));
        top_blocks.dedup_by_key(|pos| pos.x);

        for mut pos in top_blocks {
            pos.y += 1;
            let Some([bx, by]) = tf.base_pixel(pos) else {
                continue;
            };

            let fby_max = tf.base[1] as usize - tf.size[1] as usize * tf.scale;

            for dx in 0..tf.scale {
                let fbx = bx + dx;
                #[allow(clippy::needless_range_loop)]
                for fby in fby_max..=by {
                    let trail_pos = (by - fby) as f32 / (self.trail_len as f32 * tf.scale as f32);
                    let t = global_t + trail_pos;
                    let opacity = (1.0 - t) * TRAIL_OPACITY;
                    let pixel = &mut fb[[fbx, fby]];
                    *pixel = pixel.mix(color, opacity);
                }
            }
        }
    }
}

/// Animation when clearing a row.
#[derive(Debug)]
pub struct ClearAnimation {
    frame: u32,
    rows: Vec<i8>,
}
impl_animation_frame!(ClearAnimation, constants::animations::clear::DURATION);
impl ClearAnimation {
    pub fn new(rows: Vec<i8>) -> Self {
        Self { frame: 0, rows }
    }
}
impl Animation<Transform> for ClearAnimation {
    fn draw(&self, fb: &mut FrameBufferRect<'_>, tf: Transform) {
        use constants::animations::clear::*;

        let swipe_t = self.frame as f32 / (SWIPE_DURATION * FPS as f32);

        let pixel_width = tf.size[0] as usize * tf.scale;

        for &y in &self.rows {
            let Some([fbx, fby]) = tf.base_pixel(Pos { x: 0, y }) else {
                continue;
            };
            for dy in 0..tf.scale {
                for dx in 0..pixel_width {
                    let fade_t = ((swipe_t - dx as f32 / pixel_width as f32) * SWIPE_DURATION
                        / FADE_DURATION)
                        .clamp(0.0, 1.0);
                    if fade_t > 0.0 {
                        let pixel = &mut fb[[fbx + dx, fby - dy]];
                        *pixel = pixel.lighten(1.0 - fade_t).darken(fade_t);
                    }
                }
            }
        }
    }
}
