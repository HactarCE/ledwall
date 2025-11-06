use crate::{FrameBufferRect, FullInput, StaticImage};

pub trait Widget<I> {
    fn step(&mut self, _input: I) {}

    fn draw(&self, fb: &mut FrameBufferRect<'_>);
}

pub trait Activity: Widget<FullInput> {
    fn menu_image(&self) -> StaticImage;

    fn reset(&mut self);

    /// Returns whether to stay awake even if all controllers disconnect.
    fn stay_awake(&self) -> bool {
        false
    }
}
