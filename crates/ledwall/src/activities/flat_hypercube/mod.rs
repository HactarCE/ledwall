use std::time::Instant;

use flat_hypercube_logic::{Facet, Piece, Pos4, Puzzle, Turn};
use rand::SeedableRng;

mod animations;
mod constants;
mod input;

use crate::{
    Activity, AnimationFrame, ArrayVec, BLACK, FrameBufferRect, FullInput, Rgb, WHITE, Widget,
    step_opt_animation,
};

#[derive(Debug, Default)]
pub struct FlatHypercube {
    puzzle: Puzzle,

    input: PuzzleInput,

    undo_stack: Vec<Turn>,
    redo_stack: Vec<Turn>,
    last_move: Option<Turn>,

    was_scrambled: bool,
    timer_start: Option<Instant>,
    timer_end: Option<Instant>,
    show_timer: bool,

    enable_filters: bool,
    editing_filters: Option<EditingFilters>,
    filters: Filters,

    blink_anim: animations::BlinkAnimation,
    turn_anim: Option<animations::TurnAnimation>,
}

impl FlatHypercube {
    fn do_turn_with_animation(&mut self, turn: Turn) {
        if self.puzzle.do_turn(turn) {
            self.turn_anim = Some(animations::TurnAnimation::new(turn));
            if self.timer_start.is_some() && self.timer_end.is_none() && self.puzzle.is_solved() {
                self.timer_end = Some(Instant::now());
            }
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

        let any_modifiers = keys_down.lt || keys_down.rt || keys_down.plus;

        // Temporarily disable filters
        self.enable_filters ^= self.editing_filters.is_some() || !(keys_down.plus && keys_down.b);

        if keys_down.lt || keys_down.rt {
            // Reset
            if keys_pressed.x {
                self.reset();
            }

            // Scramble
            if keys_pressed.y {
                if keys_pressed.y {
                    self.reset();
                    self.puzzle
                        .scramble(&mut rand::rngs::SmallRng::from_os_rng());
                    self.was_scrambled = true;
                }
            }
        }

        if keys_down.plus {
            // Toggle timer
            if keys_pressed.a {
                self.show_timer ^= true;
            }

            // Toggle filters menu
            if keys_pressed.right {
                match self.editing_filters {
                    Some(_) => self.editing_filters = None,
                    None => self.editing_filters = Some(EditingFilters::default()),
                }
            }
        }

        if let Some(editing_filters) = &mut self.editing_filters {
            // Scroll
            if keys_down.plus && keys_pressed.up {
                editing_filters.index = editing_filters.index.saturating_sub(1);
            }
            if keys_down.plus && keys_pressed.down {
                editing_filters.index = editing_filters.index.saturating_add(1);
            }

            // Implicitly add a new filter rule, or limit the index
            while editing_filters.index >= self.filters.rules.len() {
                if self.filters.rules.try_push(FilterRule::default()).is_err() {
                    editing_filters.index = self.filters.rules.len() - 1;
                }
            }

            let current_rule = &mut self.filters.rules[editing_filters.index];

            if !any_modifiers {
                // Toggle facet
                let maybe_facet = match () {
                    _ if keys_pressed.right => Some(Facet::R),
                    _ if keys_pressed.left => Some(Facet::L),
                    _ if keys_pressed.up => Some(Facet::U),
                    _ if keys_pressed.down => Some(Facet::D),
                    _ if keys_pressed.a => Some(Facet::F),
                    _ if keys_pressed.y => Some(Facet::B),
                    _ if keys_pressed.x => Some(Facet::O),
                    _ if keys_pressed.b => Some(Facet::I),
                    _ => None,
                };

                if let Some(facet) = maybe_facet {
                    current_rule.enabled = true;

                    let facet = self.puzzle[facet.center_sticker()];
                    let bit = 1 << facet.id();

                    if current_rule.must_not_have & bit != 0 {
                        // must not have -> may have
                        current_rule.must_not_have &= !bit;
                    } else if current_rule.must_have & bit != 0 {
                        // must have -> must not have
                        current_rule.must_have &= !bit;
                        current_rule.must_not_have |= bit;
                    } else {
                        // may have -> must have
                        current_rule.must_have |= bit;
                    }
                }
            }

            // Toggle rule
            if keys_down.plus && keys_pressed.left {
                current_rule.enabled ^= true;
            }

            // Delete rule
            if keys_pressed.minus {
                self.filters.rules.remove(editing_filters.index);
                if self.filters.rules.is_empty() {
                    self.filters = Filters::default();
                }
                if editing_filters.index >= self.filters.rules.len() {
                    editing_filters.index = self.filters.rules.len() - 1;
                }
            }
        } else if !any_modifiers {
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

                if self.was_scrambled && self.timer_start.is_none() && turn.facet.is_some() {
                    self.timer_start = Some(Instant::now());
                }
            }

            // Undo
            if keys_pressed.l {
                if matches!(self.input.state, PuzzleInputState::Init) {
                    if let Some(turn) = self.undo_stack.pop() {
                        self.do_turn_with_animation(turn.inverse());
                        self.redo_stack.push(turn);
                        if self.last_move != Some(turn) {
                            self.last_move = None;
                        }
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
                        if self.last_move != Some(turn) {
                            self.last_move = None;
                        }
                    } else if let Some(turn) = self.last_move {
                        self.do_turn_with_animation(turn);
                        self.undo_stack.push(turn);
                    }
                } else {
                    self.input.state = PuzzleInputState::Init;
                }
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
        use constants::colors;

        // center
        const CX: i8 = 16;
        const CY: i8 = 16;

        const MAX_OVERLAPS: usize = 3;
        let mut overlaps_buffer = [[ArrayVec::<u8, MAX_OVERLAPS>::new(); 32]; 32];

        // Draw pieces and record stickers to `overlaps_buffer`
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
            let sticker_colors = piece
                .stickers()
                .map(|sticker| self.puzzle[sticker])
                .collect::<Vec<_>>();
            let hidden = self.enable_filters && !self.filters.has_piece(&sticker_colors);

            // Draw pieces immediately
            let [fbx, fby] = rotate_and_project(piece.pos());
            fb.set(
                fbx,
                fby,
                self.piece_color(piece)
                    .darken(if dim { colors::DARKEN_UNGRIPPED } else { 0.0 })
                    .darken(if hidden { colors::DARKEN_HIDDEN } else { 0.0 }),
            );

            // Draw stickers to an off-screen buffer for blending.
            for sticker in piece.stickers() {
                let [fbx, fby] = rotate_and_project(sticker.pos());
                overlaps_buffer[fby][fbx]
                    .push(self.puzzle[sticker].id() | ((dim as u8) << 4) | ((hidden as u8) << 5));
            }
        }

        // Draw stickers
        for (fby, row) in overlaps_buffer.into_iter().enumerate() {
            for (fbx, overlaps) in row.into_iter().enumerate() {
                if overlaps.is_empty() {
                    continue;
                }
                let iter = overlaps.iter().map(|&i| {
                    let dim = i >> 4 != 0;
                    let hidden = i >> 5 != 0;
                    let id = i & 0xF;
                    (
                        colors::STICKERS[id as usize],
                        1.0 - (1.0 - if dim { colors::DARKEN_UNGRIPPED } else { 0.0 })
                            * (1.0 - if hidden { colors::DARKEN_HIDDEN } else { 0.0 }),
                    )
                });
                let min_darken = iter
                    .clone()
                    .map(|(_color, darken)| darken)
                    .min_by(f32::total_cmp)
                    .unwrap_or(1.0);
                let overlapping_colors = Rgb::mix_multiple(
                    iter.filter(|&(_color, darken)| darken == min_darken)
                        .map(|(color, darken)| color.darken(darken)),
                );
                if min_darken == 1.0 || fb.get_mut(fbx, fby).copied() == Some(BLACK) {
                    fb.set(fbx, fby, overlapping_colors);
                }
            }
        }

        // Draw filters UI
        if let Some(editing_filters) = &self.editing_filters {
            let mut fb = fb.with_offset([0, 32]);
            let cx = 7;
            for (i, rule) in self.filters.rules.iter().enumerate() {
                let y = i * 3;
                let darken = if rule.enabled {
                    0.0
                } else {
                    colors::DARKEN_HIDDEN
                };
                for (j, facet) in Facet::ALL.into_iter().enumerate() {
                    let x = cx + j - 4;
                    fb.set(x, y, colors::STICKERS[facet.id() as usize].darken(darken));
                    let bit = 1 << facet.id();
                    let indicator_color = if rule.must_have & bit != 0 {
                        colors::FILTER_MUST_HAVE
                    } else if rule.must_not_have & bit != 0 {
                        colors::FILTER_MUST_NOT_HAVE
                    } else {
                        colors::FILTER_MAY_HAVE
                    };
                    fb.set(x, y + 1, indicator_color);
                }
                if editing_filters.index == i {
                    fb.set(cx - 6, y, WHITE);
                    fb.set(cx - 6, y + 1, WHITE);
                    fb.set(cx + 5, y, WHITE);
                    fb.set(cx + 5, y + 1, WHITE);
                }
            }

            let rule = self.filters.rules[editing_filters.index];

            draw_compass(22, 6, &mut fb, |facet, [dx, dy]| {
                let sticker_color =
                    colors::STICKERS[self.puzzle[facet.center_sticker()].id() as usize];
                let bit = 1 << facet.id();
                if rule.must_have & bit != 0 {
                    // must have
                    if dx != 1 || dy != 1 {
                        sticker_color
                    } else {
                        BLACK
                    }
                } else if rule.must_not_have & bit != 0 {
                    // must not have
                    if (dx == 1) == (dy == 1) {
                        sticker_color
                    } else {
                        BLACK
                    }
                } else {
                    // may have
                    if dx == 1 && dy == 1 {
                        sticker_color
                    } else {
                        BLACK
                    }
                }
            });
        }

        // Draw timer
        if self.show_timer {
            let timer_end = self.timer_end.unwrap_or_else(Instant::now);
            let timer_start = self.timer_start.unwrap_or(timer_end);
            let duration = timer_end.saturating_duration_since(timer_start);
            let centis = duration.subsec_millis() / 10;
            let seconds = duration.as_secs() % 60;
            let minutes = duration.as_secs() / 60;
            let text = format!("{minutes}:{seconds:02}.{centis:02}");
            let text_width = crate::text::width(&text);
            crate::text::draw(
                &text,
                &mut fb.with_offset([
                    fb.width() as isize - text_width as isize - 1,
                    fb.height() as isize - 6,
                ]),
                if self.timer_end.is_some() {
                    colors::TIMER_DONE
                } else {
                    colors::TIMER_RUNNING
                },
            );
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FilterRule {
    pub must_have: u8,
    pub must_not_have: u8,
    pub enabled: bool,
}
impl Default for FilterRule {
    fn default() -> Self {
        Self {
            must_have: 0x00,
            must_not_have: 0xFF,
            enabled: true,
        }
    }
}
impl FilterRule {
    pub fn has_piece(self, stickers: &[Facet]) -> bool {
        self.enabled
            && Facet::ALL.iter().all(|&f| {
                (self.must_have & (1 << f.id()) == 0 || stickers.contains(&f))
                    && (self.must_not_have & (1 << f.id()) == 0 || !stickers.contains(&f))
            })
    }
}

#[derive(Debug)]
pub struct Filters {
    pub rules: ArrayVec<FilterRule, 8>,
}
impl Default for Filters {
    fn default() -> Self {
        let mut rules = ArrayVec::new();
        rules.push(FilterRule {
            must_have: 0x00,
            must_not_have: 0x00,
            enabled: true,
        });
        Self { rules }
    }
}
impl Filters {
    pub fn has_piece(&self, stickers: &[Facet]) -> bool {
        self.rules.iter().any(|rule| rule.has_piece(stickers))
    }
}

#[derive(Debug, Default)]
pub struct EditingFilters {
    pub index: usize,
}

fn draw_compass(
    cx: usize,
    cy: usize,
    fb: &mut FrameBufferRect<'_>,
    mut color_fn: impl FnMut(Facet, [i8; 2]) -> Rgb,
) {
    for facet in Facet::ALL {
        let v = facet.center_piece().pos();
        for dy in 0..3 {
            for dx in 0..3 {
                fb.set(
                    (cx as i8 + (v.0[0] + v.0[2] * 2) * 3 + dx) as usize,
                    (cy as i8 - (v.0[1] + v.0[3] * 2) * 3 + dy) as usize,
                    color_fn(facet, [dx, dy]),
                );
            }
        }
    }
}
