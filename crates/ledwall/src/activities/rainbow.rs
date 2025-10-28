use std::time::Instant;

use crate::{Activity, FrameBufferRect, Input, Rgb, Widget};

#[derive(Debug)]
pub struct Rainbow {
    start_time: Instant,
}

impl Rainbow {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
        }
    }
}

impl Widget<Input> for Rainbow {
    fn draw(&mut self, fb: &mut FrameBufferRect<'_>) {
        let t = self.start_time.elapsed().as_secs_f64() / 2.0;
        fb.fill_with_fn(|[x, y], _| {
            let color = colorous::RAINBOW
                .eval_continuous(((x as f64 + y as f64 * 2.0) / 64.0 - t).rem_euclid(1.0));
            Rgb(color.as_array().map(|x| x))
        });
    }
}

impl Activity for Rainbow {
    fn menu_image(&self) -> crate::StaticImage {
        include_rgba_image!("menu/rainbow.rgba")
    }
}
