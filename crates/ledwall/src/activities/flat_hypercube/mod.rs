use flat_hypercube_logic::{Facet, Piece, Pos4, Puzzle, Turn};
use rand::SeedableRng;

mod animations;
mod constants;
mod input;

use crate::{
    Activity, AnimationFrame, ArrayVec, FrameBufferRect, FullInput, Rgb, Widget, step_opt_animation,
};

#[derive(Debug, Default)]
pub struct FlatHypercube {
    puzzle: Puzzle,

    input: PuzzleInput,

    undo_stack: Vec<Turn>,
    redo_stack: Vec<Turn>,
    last_move: Option<Turn>,

    blink_anim: animations::BlinkAnimation,
    turn_anim: Option<animations::TurnAnimation>,
}

impl FlatHypercube {
    fn do_turn_with_animation(&mut self, turn: Turn) {
        if self.puzzle.do_turn(turn) {
            self.turn_anim = Some(animations::TurnAnimation::new(turn));
        }
    }

    fn piece_color(&self, piece: Piece) -> Rgb {
        use PuzzleInputState as State;

        let is_selected = match self.input.state {
            State::Init => false,
            State::Rotate1 | State::Rotate2(_) => true,
            State::Turn1(facet) | State::Turn2(facet, _) => facet.has_piece(piece),
        };
        let is_blinking = is_selected
            && match self.input.state {
                State::Init | State::Rotate1 | State::Turn1(_) => false,
                State::Rotate2(from) | State::Turn2(_, from) => from.has_piece(piece),
            };

        let mut color = if is_selected {
            constants::colors::INTERNALS_SELECTED
        } else {
            constants::colors::INTERNALS
        };

        if is_blinking {
            color = color.lighten(1.0 - crate::smooth_sine(self.blink_anim.t()));
        }

        if let Some(red_flash) = &self.input.red_flash_anim
            && red_flash.facet1.has_piece(piece)
            && red_flash.facet2.is_none_or(|f| f.has_piece(piece))
        {
            color = constants::colors::RED_FLASH.mix(color, red_flash.t());
        }

        color
    }
}

impl Widget<FullInput> for FlatHypercube {
    fn step(&mut self, input: FullInput) {
        let keys_down = input.any().current;
        let keys_pressed = input.any().pressed();

        if keys_down.plus {
            if keys_pressed.x {
                self.reset();
            }
            if keys_pressed.y {
                self.reset();
                self.puzzle
                    .scramble(&mut rand::rngs::SmallRng::from_os_rng());
            }
        } else {
            // Input turn
            let maybe_turn = match () {
                _ if keys_pressed.minus => self.input.input_rotate(),
                _ if keys_pressed.right => self.input.input_facet(Facet::R),
                _ if keys_pressed.left => self.input.input_facet(Facet::L),
                _ if keys_pressed.up => self.input.input_facet(Facet::U),
                _ if keys_pressed.down => self.input.input_facet(Facet::D),
                _ if keys_pressed.a => self.input.input_facet(Facet::F),
                _ if keys_pressed.y => self.input.input_facet(Facet::B),
                _ if keys_pressed.x => self.input.input_facet(Facet::O),
                _ if keys_pressed.b => self.input.input_facet(Facet::I),
                _ => None,
            };
            if let Some(turn) = maybe_turn {
                self.do_turn_with_animation(turn);
                self.undo_stack.push(turn);
                self.redo_stack.clear();
                self.last_move = Some(turn);
            }
        }

        // Undo
        if keys_pressed.l {
            if matches!(self.input.state, PuzzleInputState::Init) {
                if let Some(turn) = self.undo_stack.pop() {
                    self.do_turn_with_animation(turn.inverse());
                    self.redo_stack.push(turn);
                    self.last_move = None;
                }
            } else {
                self.input.state = PuzzleInputState::Init;
            }
        }

        // Redo
        if keys_pressed.r {
            if matches!(self.input.state, PuzzleInputState::Init) {
                if let Some(turn) = self.redo_stack.pop() {
                    self.do_turn_with_animation(turn);
                    self.undo_stack.push(turn);
                    self.last_move = None;
                } else if let Some(turn) = self.last_move {
                    self.do_turn_with_animation(turn);
                    self.undo_stack.push(turn);
                }
            } else {
                self.input.state = PuzzleInputState::Init;
            }
        }

        step_opt_animation(&mut self.input.red_flash_anim);
        step_opt_animation(&mut self.turn_anim);

        match self.input.state {
            PuzzleInputState::Init | PuzzleInputState::Rotate1 | PuzzleInputState::Turn1(_) => {
                self.blink_anim.reset();
            }
            PuzzleInputState::Rotate2(_) | PuzzleInputState::Turn2(_, _) => self.blink_anim.step(),
        }
    }

    fn draw(&self, fb: &mut FrameBufferRect<'_>) {
        // center
        const CX: i8 = 16;
        const CY: i8 = 16;

        const MAX_OVERLAPS: usize = 3;
        let mut overlaps_buffer = [[ArrayVec::<u8, MAX_OVERLAPS>::new(); 32]; 32];

        for piece in Piece::iter_all() {
            let rotate_and_project = |pos: Pos4| {
                // Rotate
                let rotated_pos = if let Some(anim) = &self.turn_anim {
                    anim.modify(pos)
                } else {
                    pos.into()
                };

                // Project
                let [x, y] = rotated_pos.integer_display_pos();
                [(CX + x) as usize, (CY - y) as usize]
            };

            let dim = match self.input.state {
                PuzzleInputState::Init | PuzzleInputState::Rotate1 => false,
                PuzzleInputState::Rotate2(facet)
                | PuzzleInputState::Turn1(facet)
                | PuzzleInputState::Turn2(facet, _) => !facet.has_piece(piece),
            };
            let darken = if dim {
                constants::colors::DARKEN_UNGRIPPED
            } else {
                0.0
            };

            // Draw pieces immediately
            let [fbx, fby] = rotate_and_project(piece.pos());
            fb.set(fbx, fby, self.piece_color(piece).darken(darken));

            // Draw stickers to an off-screen buffer for blending.
            for sticker in piece.stickers() {
                let [fbx, fby] = rotate_and_project(sticker.pos());
                overlaps_buffer[fby][fbx].push(self.puzzle[sticker].id() + ((dim as u8) << 4));
            }
        }

        for (fby, row) in overlaps_buffer.into_iter().enumerate() {
            for (fbx, overlaps) in row.into_iter().enumerate() {
                if overlaps.is_empty() {
                    continue;
                }
                let overlapping_colors = Rgb::mix_multiple(overlaps.iter().map(|&i| {
                    let dim = i >> 4 != 0;
                    let darken = if dim {
                        constants::colors::DARKEN_UNGRIPPED
                    } else {
                        0.0
                    };
                    let id = i & 0xF;
                    constants::colors::STICKERS[id as usize].darken(darken)
                }));
                fb.set(fbx, fby, overlapping_colors);
            }
        }
    }
}

impl Activity for FlatHypercube {
    fn reset(&mut self) {
        *self = Self::default();
    }

    fn menu_image(&self) -> crate::StaticImage {
        include_rgba_image!("activities/flat_hypercube.rgba")
    }
}

#[derive(Debug, Default)]
pub struct PuzzleInput {
    state: PuzzleInputState,
    red_flash_anim: Option<animations::RedFlashAnimation>,
}
impl PuzzleInput {
    pub fn input_rotate(&mut self) -> Option<Turn> {
        self.state.input_rotate().unwrap_or_else(|anim| {
            self.red_flash_anim = Some(anim);
            None
        })
    }

    pub fn input_facet(&mut self, facet: Facet) -> Option<Turn> {
        self.state.input_facet(facet).unwrap_or_else(|anim| {
            self.red_flash_anim = Some(anim);
            None
        })
    }
}

/// State machine for puzzle input.
#[derive(Debug, Default)]
pub enum PuzzleInputState {
    #[default]
    Init,
    Rotate1,
    Rotate2(Facet),
    Turn1(Facet),
    Turn2(Facet, Facet),
}
impl PuzzleInputState {
    pub fn input_rotate(&mut self) -> Result<Option<Turn>, animations::RedFlashAnimation> {
        match self {
            Self::Init => {
                *self = Self::Rotate1;
                Ok(None)
            }
            Self::Rotate1 | Self::Rotate2(_) | Self::Turn1(_) | Self::Turn2(_, _) => {
                *self = Self::Init;
                Ok(None)
            }
        }
    }

    pub fn input_facet(
        &mut self,
        new_facet: Facet,
    ) -> Result<Option<Turn>, animations::RedFlashAnimation> {
        match self {
            Self::Rotate2(f1) | Self::Turn1(f1) | Self::Turn2(f1, _) if *f1 == new_facet => {
                *self = Self::Init;
                Ok(None)
            }
            Self::Turn2(f1, f2) if *f2 == new_facet => {
                *self = Self::Turn1(*f1);
                Ok(None)
            }

            Self::Rotate2(f1) | Self::Turn1(f1) | Self::Turn2(f1, _)
                if f1.axis == new_facet.axis =>
            {
                Err(animations::RedFlashAnimation::new(new_facet, None))
            }
            Self::Turn2(f1, f2) if f2.axis == new_facet.axis => {
                Err(animations::RedFlashAnimation::new(new_facet, Some(*f1)))
            }

            Self::Init => {
                *self = Self::Turn1(new_facet);
                Ok(None)
            }
            Self::Rotate1 => {
                *self = Self::Rotate2(new_facet);
                Ok(None)
            }
            Self::Turn1(facet) => {
                *self = Self::Turn2(*facet, new_facet);
                Ok(None)
            }

            &mut Self::Rotate2(from) => {
                *self = Self::Init;
                Ok(Some(Turn {
                    facet: None,
                    from,
                    to: new_facet,
                }))
            }
            &mut Self::Turn2(facet, from) => {
                *self = Self::Init;
                Ok(Some(Turn {
                    facet: Some(facet),
                    from,
                    to: new_facet,
                }))
            }
        }
    }
}
