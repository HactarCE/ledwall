use crate::{Animation, AnimationFrame, FrameBufferRect, WHITE};

#[derive(Debug, Default)]
pub struct LoadingAnimation {
    frame: u32,
}

impl_animation_frame!(LoadingAnimation, 1.0);

impl Animation for LoadingAnimation {
    fn draw(&self, fb: &mut FrameBufferRect<'_>, _data: ()) {
        let mut fb = fb.centered_square();
        let rx = fb.width() as f32 / 2.0;
        let ry = fb.height() as f32 / 2.0;
        let start_deg = self.t() * 360.0;
        for a in 0..360 {
            let (sin, cos) = (start_deg - a as f32).to_radians().sin_cos();
            fb.set(
                (rx + rx * cos) as usize,
                (ry + ry * sin) as usize,
                WHITE.darken(a as f32 / 360.0),
            );
        }
    }
}
