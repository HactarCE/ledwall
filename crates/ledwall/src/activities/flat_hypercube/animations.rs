use std::f32::consts::PI;

use flat_hypercube_logic::{Facet, FloatPos4, Pos4, Turn};

use crate::AnimationFrame;

use super::constants;

#[derive(Debug, Default)]
pub struct BlinkAnimation {
    frame: u32,
}
impl_animation_frame!(BlinkAnimation, constants::animations::blink::DURATION);
impl BlinkAnimation {
    pub fn reset(&mut self) {
        self.frame = 0;
    }

    pub fn step(&mut self) {
        self.frame += 1;
        if self.t() >= 1.0 {
            self.reset();
        }
    }
}

#[derive(Debug)]
pub struct RedFlashAnimation {
    frame: u32,
    pub facet1: Facet,
    pub facet2: Option<Facet>,
}
impl_animation_frame!(
    RedFlashAnimation,
    constants::animations::red_flash::DURATION,
);
impl RedFlashAnimation {
    pub fn new(facet1: Facet, facet2: Option<Facet>) -> Self {
        Self {
            frame: 0,
            facet1,
            facet2,
        }
    }
}

#[derive(Debug)]
pub struct TurnAnimation {
    frame: u32,
    pub turn: Turn,
}
impl_animation_frame!(TurnAnimation, constants::animations::turn::DURATION);
impl TurnAnimation {
    pub fn new(turn: Turn) -> Self {
        Self { frame: 0, turn }
    }

    pub fn modify(&self, pos: Pos4) -> FloatPos4 {
        let t = (-1.0 - (self.t() * PI).cos()) / 2.0; // -1 to 0

        let is_affected = match self.turn.facet {
            Some(facet) => facet.has_pos(pos),
            None => true,
        };
        let float_pos: FloatPos4 = pos.into();
        if is_affected {
            float_pos
                .rot(self.turn.from, self.turn.to, t)
                .unwrap_or(float_pos)
        } else {
            float_pos
        }
    }
}
