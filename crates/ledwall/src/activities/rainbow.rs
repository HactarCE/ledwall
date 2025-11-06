use crate::{Activity, FPS, FrameBufferRect, FullInput, Rgb, Widget};

pub const DURATION: f32 = 2.0; // seconds

#[derive(Debug, Default)]
pub struct Rainbow {
    frame: usize,
}

impl Widget<FullInput> for Rainbow {
    fn step(&mut self, _input: FullInput) {
        self.frame += 1;
        if self.frame == (DURATION * FPS as f32) as usize {
            self.frame = 0;
        }
    }

    fn draw(&self, fb: &mut FrameBufferRect<'_>) {
        let t = self.frame as f64 / FPS as f64 / DURATION as f64;
        fb.fill_with_fn(|[x, y], _| {
            let color = colorous::RAINBOW
                .eval_continuous(((x as f64 + y as f64 * 2.0) / 64.0 - t).rem_euclid(1.0));
            Rgb(color.as_array().map(|x| x))
        });
    }
}

impl Activity for Rainbow {
    fn reset(&mut self) {
        *self = Self::default();
    }

    fn menu_image(&self) -> crate::StaticImage {
        include_rgba_image!("menu/rainbow.rgba")
    }
}
