use std::collections::HashSet;

use rand::{Rng, SeedableRng};

use crate::{Activity, BLACK, FPS, FullInput, HEIGHT, Rgb, WHITE, WIDTH, Widget};

const TRAIL_BRIGHTNESS: f32 = 0.75;
const TRAIL_LIMIT: u8 = 30;

const FAST_RATE: usize = 1;
const NORMAL_RATE: usize = 3;
const SLOW_RATE: usize = 10;

const RESET_TIME: usize = FPS / NORMAL_RATE * 2; // 2 seconds

pub struct Life {
    /// State of each cell
    ///
    /// - 0 = dead
    /// - 1 = alive
    /// - 2.. = frames since last alive
    cells: [[u8; WIDTH]; HEIGHT],

    frame: usize,

    rainbow: super::rainbow::Rainbow,

    history: HashSet<[[u8; WIDTH]; HEIGHT]>,
    reset_timer: Option<usize>,
}

impl Default for Life {
    fn default() -> Self {
        Self {
            cells: [[0; WIDTH]; HEIGHT],

            frame: 0,

            rainbow: super::rainbow::Rainbow::default(),

            history: HashSet::new(),
            reset_timer: None,
        }
    }
}

impl Activity for Life {
    fn reset(&mut self) {
        *self = Self::default();
    }

    fn menu_image(&self) -> crate::StaticImage {
        include_rgba_image!("menu/life.rgba")
    }
}

impl Widget<FullInput> for Life {
    fn step(&mut self, input: FullInput) {
        self.rainbow.step(input);

        let pressed = input.any().pressed();
        let held = input.any().current;

        if pressed.y {
            self.reset_random();
        }

        if pressed.x {
            self.reset();
        }

        let frames_per_step = match (held.l, held.r) {
            (true, false) => SLOW_RATE,
            (false, true) => FAST_RATE,
            _ => NORMAL_RATE,
        };

        self.frame += 1;
        if self.frame < frames_per_step {
            return;
        }
        self.frame = 0;

        let mut new_cells = self.cells;
        for y in 0..64_i8 {
            for x in 0..32_i8 {
                let center = self.cell(x, y);
                let new_state = transition(center, {
                    (-1..=1)
                        .flat_map(|dy| (-1..=1).map(move |dx| [dx, dy]))
                        .filter(|[dx, dy]| self.cell(x + dx, y + dy))
                        .count()
                        - center as usize
                });
                let state = &mut new_cells[y as usize][x as usize];
                if new_state {
                    *state = 1;
                } else if *state != 0 {
                    *state = state.wrapping_add(1);
                }
            }
        }

        self.cells = new_cells;

        if let Some(reset_timer) = &mut self.reset_timer {
            if *reset_timer > 0 {
                *reset_timer -= 1;
                if *reset_timer == 0 {
                    self.reset_random();
                }
            } else if !self.history.insert(self.cells) {
                *reset_timer = RESET_TIME;
            }
        }
    }

    fn draw(&self, fb: &mut crate::FrameBufferRect<'_>) {
        self.rainbow.draw(fb);
        fb.fill_with_fn(|[x, y], rainbow_color| get_color(self.cells[y][x], rainbow_color));
    }
}

impl Life {
    fn cell(&self, x: i8, y: i8) -> bool {
        self.cells[y.rem_euclid(HEIGHT as i8) as usize][x.rem_euclid(WIDTH as i8) as usize] == 1
    }

    fn reset(&mut self) {
        self.history.clear();
        self.cells = [[0; WIDTH]; HEIGHT];
        self.reset_timer = None;
    }
    fn reset_random(&mut self) {
        self.history.clear();
        self.cells = rand::rngs::SmallRng::from_os_rng()
            .random::<[[bool; WIDTH]; HEIGHT]>()
            .map(|row| row.map(|cell| cell as u8));
        self.reset_timer = Some(0);
    }
}

fn transition(current: bool, neighbor_count: usize) -> bool {
    if current {
        neighbor_count == 2 || neighbor_count == 3
    } else {
        neighbor_count == 3
    }
}

fn get_color(state: u8, trail_color: Rgb) -> Rgb {
    match state {
        0 => BLACK,
        1 => WHITE,
        2.. => trail_color
            .darken(1.0 - TRAIL_BRIGHTNESS)
            .darken((state - 2) as f32 / TRAIL_LIMIT as f32),
    }
}
