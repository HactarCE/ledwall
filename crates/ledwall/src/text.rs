use std::borrow::Cow;

use crate::{Font, FrameBufferRect, Tint, TintFn, Widget};

#[derive(Debug, Clone)]
pub struct Text<'a, T: Tint> {
    font: &'a Font,
    s: Cow<'a, str>,
    tint: T,
}
impl<'a, T: Tint> Text<'a, T> {
    pub fn new(s: impl Into<Cow<'a, str>>, font: &'a Font, tint: T) -> Self {
        Self {
            font,
            s: s.into(),
            tint,
        }
    }

    pub fn width(&self) -> usize {
        self.s
            .chars()
            .map(|c| self.font.get(c))
            .map(|img| img.width() + 1)
            .sum::<usize>()
            - 1
    }
}

impl<'a, T: Tint> Widget for Text<'a, T> {
    fn draw(&self, fb: &mut FrameBufferRect<'_>) {
        let mut x = 0;
        for img in self.s.chars().map(|c| self.font.get(c)) {
            img.draw(
                &mut fb.with_offset([x as isize, 0]),
                TintFn(|[dx, dy], color| self.tint.eval_tint([x + dx, dy], color)),
            );
            x += img.width() + 1;
        }
    }
}
