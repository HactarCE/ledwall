use crate::{FrameBufferRect, FullInput, Rgb, StaticImage};

pub trait Widget<I = ()> {
    fn step(&mut self, _input: I) {}

    fn draw(&self, fb: &mut FrameBufferRect<'_>);
}

pub trait Activity: Widget<FullInput> {
    fn menu_image(&self) -> StaticImage {
        include_rgba_image!("activities/unknown.rgba")
    }

    fn reset(&mut self) {}

    /// Returns whether to stay awake even if all controllers disconnect.
    fn stay_awake(&self) -> bool {
        false
    }
}

/// Function that can be used to modify a image or region in a framebuffer at
/// each pixel.
///
/// For basic alpha blending with no tint, use `()`.
pub trait Tint {
    fn eval_tint(&self, pos: [usize; 2], color: Rgb) -> Rgb;

    fn at(self, [dx, dy]: [usize; 2]) -> impl Tint
    where
        Self: Sized,
    {
        TintFn(move |[x, y], color| self.eval_tint([x + dx, y + dy], color))
    }
}

impl Tint for () {
    fn eval_tint(&self, _pos: [usize; 2], color: Rgb) -> Rgb {
        color
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Darken(pub f32);
impl Tint for Darken {
    fn eval_tint(&self, _pos: [usize; 2], color: Rgb) -> Rgb {
        color.darken(self.0)
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Lighten(pub f32);
impl Tint for Lighten {
    fn eval_tint(&self, _pos: [usize; 2], color: Rgb) -> Rgb {
        color.lighten(self.0)
    }
}

impl Tint for Rgb {
    fn eval_tint(&self, _pos: [usize; 2], _color: Rgb) -> Rgb {
        *self
    }
}

#[derive(Copy, Clone)]
pub struct TintFn<F: Fn([usize; 2], Rgb) -> Rgb>(pub F);
impl<F: Fn([usize; 2], Rgb) -> Rgb> Tint for TintFn<F> {
    fn eval_tint(&self, pos: [usize; 2], color: Rgb) -> Rgb {
        (self.0)(pos, color)
    }
}

/// Function for blending a background color (`bg`) with a foreground color
/// (`fg`) using an alpha value (`alpha`).
///
/// When constructed from a `Tint`, the tint applies to the image pixels and
/// then is alpha-blended onto the framebuffer.
///
/// For basic alpha blending with no tint, use `()`.
pub trait Blend {
    fn eval_blend(&self, pos: [usize; 2], bg: Rgb, fg: Rgb, alpha: u8) -> Rgb;
}

impl<T: Tint> Blend for T {
    fn eval_blend(&self, pos: [usize; 2], bg: Rgb, fg: Rgb, alpha: u8) -> Rgb {
        bg.mix(self.eval_tint(pos, fg), alpha as f32 / 255.0)
    }
}

#[derive(Copy, Clone)]
pub struct BlendFn<F: Fn([usize; 2], Rgb, Rgb, u8) -> Rgb>(pub F);
impl<F: Fn([usize; 2], Rgb, Rgb, u8) -> Rgb> Blend for BlendFn<F> {
    fn eval_blend(&self, pos: [usize; 2], bg: Rgb, fg: Rgb, alpha: u8) -> Rgb {
        (self.0)(pos, bg, fg, alpha)
    }
}
