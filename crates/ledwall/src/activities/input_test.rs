use crate::{Activity, FrameBufferRect, FullInput, Rgb, Widget};

#[derive(Debug, Default)]
pub struct InputTest {
    input: FullInput,
}

impl Widget<FullInput> for InputTest {
    fn step(&mut self, input: FullInput) {
        self.input = input;
    }

    fn draw(&self, fb: &mut FrameBufferRect<'_>) {
        for (i, controller) in [self.input.blue, self.input.green].iter().enumerate() {
            let x0 = i * 8;
            if let Some(c) = controller {
                fb.set(x0 + 0, 0, color(c.current.up));
                fb.set(x0 + 1, 0, color(c.current.down));
                fb.set(x0 + 2, 0, color(c.current.left));
                fb.set(x0 + 3, 0, color(c.current.right));
                fb.set(x0 + 0, 1, color(c.current.a));
                fb.set(x0 + 1, 1, color(c.current.b));
                fb.set(x0 + 2, 1, color(c.current.x));
                fb.set(x0 + 3, 1, color(c.current.y));
                fb.set(x0 + 0, 2, color(c.current.l));
                fb.set(x0 + 1, 2, color(c.current.r));
                fb.set(x0 + 2, 2, color(c.current.lt));
                fb.set(x0 + 3, 2, color(c.current.rt));
                fb.set(x0 + 0, 3, color(c.current.plus));
                fb.set(x0 + 1, 3, color(c.current.minus));
                fb.set(x0 + 2, 3, color(c.current.star));
                fb.set(x0 + 3, 3, color(c.current.heart));
            }
        }
    }
}

impl Activity for InputTest {
    fn menu_image(&self) -> crate::StaticImage {
        include_rgba_image!("activities/input_test.rgba")
    }

    fn reset(&mut self) {}

    fn stay_awake(&self) -> bool {
        false
    }
}

fn color(state: bool) -> Rgb {
    Rgb([if state { 255 } else { 50 }, 50, 50])
}
