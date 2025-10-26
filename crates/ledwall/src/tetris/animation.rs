use tetris_logic::{FallingPiece, Pos};

use super::{Transform, colors, constants};
use crate::{FPS, FrameBuffer};

use private::AnimationFrame;
mod private {
    pub trait AnimationFrame {
        /// Animation duration in seconds.
        const DURATION: f32;

        /// Returns the frame number of the animation.
        fn frame_number(&self) -> u32;
        /// Returns a mutable reference to the frame number of the animation.
        fn frame_number_mut(&mut self) -> &mut u32;

        /// Returns the progress into the animation.
        fn t(&self) -> f32 {
            self.frame_number() as f32 / (Self::DURATION * crate::FPS as f32)
        }
    }
}
macro_rules! impl_animation_frame {
    ($ty:ty, $duration:expr) => {
        impl private::AnimationFrame for $ty {
            const DURATION: f32 = $duration;

            fn frame_number(&self) -> u32 {
                self.frame
            }
            fn frame_number_mut(&mut self) -> &mut u32 {
                &mut self.frame
            }
        }
    };
}

pub trait Animation: AnimationFrame {
    fn draw(&self, fb: &mut FrameBuffer, tf: Transform);
}

pub fn step_opt<A: Animation>(opt_anim: &mut Option<A>) {
    opt_anim.take_if(|anim| {
        *anim.frame_number_mut() += 1;
        anim.t() > 1.0
    });
}

pub fn draw_opt(opt_anim: &Option<impl Animation>, fb: &mut FrameBuffer, tf: Transform) {
    if let Some(anim) = opt_anim {
        anim.draw(fb, tf);
    }
}

/// Animation when a piece locks into place.
pub struct Lock {
    frame: u32,
    locked_piece: FallingPiece<u64>,
}
impl_animation_frame!(Lock, constants::animations::lock::DURATION);
impl Lock {
    pub fn new(locked_piece: FallingPiece<u64>) -> Self {
        Self {
            frame: 0,
            locked_piece,
        }
    }
}
impl Animation for Lock {
    fn draw(&self, fb: &mut FrameBuffer, tf: Transform) {
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
pub struct HardDrop {
    frame: u32,
    trail_len: i8,
    end_piece: FallingPiece<u64>,
}
impl_animation_frame!(HardDrop, constants::animations::hard_drop::DURATION);
impl HardDrop {
    pub fn new(trail_len: i8, end_piece: FallingPiece<u64>) -> Self {
        Self {
            frame: 0,
            trail_len,
            end_piece,
        }
    }
}
impl Animation for HardDrop {
    fn draw(&self, fb: &mut FrameBuffer, tf: Transform) {
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
                    let pixel = &mut fb[fby][fbx];
                    *pixel = pixel.mix(color, opacity);
                }
            }
        }
    }
}

/// Animation when clearing a row.
pub struct Clear {
    frame: u32,
    rows: Vec<i8>,
}
impl_animation_frame!(Clear, constants::animations::clear::DURATION);
impl Clear {
    pub fn new(rows: Vec<i8>) -> Self {
        Self { frame: 0, rows }
    }
}
impl Animation for Clear {
    fn draw(&self, fb: &mut FrameBuffer, tf: Transform) {
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
                        let pixel = &mut fb[fby - dy][fbx + dx];
                        *pixel = pixel.lighten(1.0 - fade_t).darken(fade_t);
                    }
                }
            }
        }
    }
}
