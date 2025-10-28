use crate::{FrameBufferRect, Input, StaticImage};

pub trait Widget<I> {
    fn step(&mut self, _input: I) {}

    fn draw(&mut self, fb: &mut FrameBufferRect<'_>);
}

pub trait Activity: Widget<Input> {
    fn menu_image(&self) -> StaticImage;

    /// Returns whether to stay awake even if all controllers disconnect.
    fn stay_awake(&self) -> bool {
        false
    }
}
