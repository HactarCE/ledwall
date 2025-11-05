use std::ops::RangeInclusive;

use crate::{
    AnimationFrame, FrameBufferRect, KeyRepeat, Rgb, Widget, map_range, step_opt_animation,
};

const BRIGHT_DURATION: f32 = 0.25;
const FADE_START: f32 = BRIGHT_DURATION;
const FADE_DURATION: f32 = 0.125;
const TOTAL_ANIMATION_DURATION: f32 = FADE_START + FADE_DURATION;

const LIGHTEN_FLASH: f32 = 0.2;
const DARKEN_FILLED: f32 = 0.2;
const DARKEN_EMPTY: f32 = 1.0;

pub struct Slider {
    /// Value from 0 to 20.
    value: u8,
    range: RangeInclusive<u8>,
    pub color: Rgb,
    decrement: KeyRepeat,
    increment: KeyRepeat,
    flash: Option<FlashAnimation>,
}

impl Slider {
    pub fn new(default: u8, range: RangeInclusive<u8>, color: Rgb) -> Self {
        Self {
            value: default.clamp(*range.start(), *range.end()),
            range,
            color,
            increment: KeyRepeat::new(),
            decrement: KeyRepeat::new(),
            flash: None,
        }
    }

    pub fn get(&self) -> u8 {
        self.value
    }
    pub fn trigger_animation(&mut self) {
        self.flash = Some(FlashAnimation { frame: 0 });
    }
    pub fn set(&mut self, new_value: u8) {
        self.value = new_value.clamp(*self.range.start(), *self.range.end());
        self.trigger_animation();
    }
}

impl Widget<[bool; 2]> for Slider {
    fn step(&mut self, [decrement, increment]: [bool; 2]) {
        if decrement || increment {
            self.trigger_animation();
        }
        if self.decrement.update(decrement && !increment) {
            self.set(self.get().saturating_sub(1));
        }
        if self.increment.update(increment && !decrement) {
            self.set(self.get().saturating_add(1));
        }
    }

    fn draw(&mut self, fb: &mut FrameBufferRect<'_>) {
        step_opt_animation(&mut self.flash);
        let t = match &self.flash {
            Some(flash) => map_range(
                flash.t() * TOTAL_ANIMATION_DURATION - FADE_START,
                0.0..FADE_DURATION,
                -1.0..1.0,
            ),
            None => 1.0,
        };

        let filled_width = ((fb.width() as f32 * self.value as f32 / *self.range.end() as f32)
            as usize)
            .clamp(0, fb.width());

        fb.with_size([filled_width, fb.height()]).fill(
            self.color
                .lighten(map_range(t, -1.0..0.0, LIGHTEN_FLASH..0.0))
                .darken(map_range(t, 0.0..1.0, 0.0..DARKEN_FILLED)),
        );

        fb.with_offset([filled_width as isize, 0])
            .fill(self.color.darken(DARKEN_EMPTY));
    }
}

struct FlashAnimation {
    frame: u32,
}
impl_animation_frame!(FlashAnimation, TOTAL_ANIMATION_DURATION);
