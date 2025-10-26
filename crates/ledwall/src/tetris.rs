use rand::SeedableRng;
use tetris_logic::{FallingPiece, FrameInput, Pos, Tetromino};

use crate::{BLACK, FPS, FrameBuffer, Input, Rgb};

const BORDER: Rgb = rgb(0x666666);
const BACKGROUND: Rgb = BLACK;

const ROW_CLEAR_TIME: u32 = 30;
const HARD_DROP_TIME: u32 = 30;

pub struct Tetris {
    game: tetris_logic::Game<u64>,
    row_clear_anim: Option<(Vec<i8>, u32)>,
    hard_drop_anim: Option<(FallingPiece<u64>, u32)>,
}

impl Default for Tetris {
    fn default() -> Self {
        Self {
            game: tetris_logic::Game::new(
                tetris_logic::Config {
                    das: Some(tetris_logic::Das {
                        delay: FPS as u64 / 3,
                        arr: FPS as u64 / 10,
                    }),
                    ..Default::default()
                },
                0,
                Box::new(rand::rngs::SmallRng::from_os_rng()),
            ),
            row_clear_anim: None,
            hard_drop_anim: None,
        }
    }
}

impl Tetris {
    pub fn step(&mut self, input: Input) {
        if let Some((_piece, frame)) = &mut self.hard_drop_anim {
            *frame += 1;
            if *frame > HARD_DROP_TIME {
                self.hard_drop_anim = None;
            }
        }

        if let Some((_rows_cleared, frame)) = &mut self.row_clear_anim {
            *frame += 1;
            if *frame <= ROW_CLEAR_TIME {
                return;
            } else {
                self.row_clear_anim = None;
            }
        }

        let result = self.game.step(
            1,
            FrameInput {
                left: input.left,
                right: input.right,
                soft_drop: input.down,
                hard_drop: input.up,
                rot_cw: input.a || input.y,
                rot_ccw: input.b,
                rot_180: input.x,
                hold: input.l || input.r || input.lt || input.rt,
            },
        );

        if let Ok(output) = &result {
            if let Some(rows_cleared) = &output.rows_cleared {
                self.row_clear_anim = Some((rows_cleared.clone(), 0));
            }
            if output.hard_drop.is_some()
                && let Some(locked_piece) = output.locked_piece
            {
                self.hard_drop_anim = Some((locked_piece, 0));
            }
        }
    }

    pub fn draw(&mut self, frame_buffer: &mut FrameBuffer) {
        let playfield = self.game.playfield();

        // Draw static blocks
        for y in 0..self.game.config().height as i8 {
            for x in 0..self.game.config().width as i8 {
                let pos = Pos { x, y };
                let color = block_color(playfield.get(pos).flatten());
                draw_big_block(frame_buffer, [0, 0], pos, color);
            }
        }

        // Draw border
        let w = self.game.config().width as i8 * 2;
        let h = self.game.config().height as i8 * 2;
        for y in 0..=h {
            draw_small_block(frame_buffer, [0, 0], Pos { x: w, y }, BORDER);
        }
        for x in 0..w {
            draw_small_block(frame_buffer, [0, 0], Pos { x, y: h }, BORDER);
        }

        let falling_piece = self.game.falling_piece();
        let falling_color = block_color(Some(falling_piece.piece));

        // Draw ghost
        if let Some(ghost_pos) = self.game.ghost_piece_pos() {
            for pos in falling_piece
                .piece
                .coordinates_at(falling_piece.rot, ghost_pos)
            {
                draw_big_block(frame_buffer, [0, 0], pos, dim(falling_color));
            }
        }

        // Draw falling block
        for pos in falling_piece
            .piece
            .coordinates_at(falling_piece.rot, falling_piece.pos)
        {
            draw_big_block(frame_buffer, [0, 0], pos, falling_color);
        }

        // Draw next blocks
        let piece = self.game.queue().nth_next_piece(0);
        let color = block_color(Some(piece));
        for offset in piece.coordinates() {
            draw_big_block(frame_buffer, [22, 28], Pos::new(1, 1) + offset, color);
        }
        let piece = self.game.queue().nth_next_piece(1);
        let color = block_color(Some(piece));
        for offset in piece.coordinates() {
            draw_small_block(frame_buffer, [24, 23], Pos::new(1, 1) + offset, color);
        }
        let piece = self.game.queue().nth_next_piece(2);
        let color = block_color(Some(piece));
        for offset in piece.coordinates() {
            draw_small_block(frame_buffer, [24, 18], Pos::new(1, 1) + offset, color);
        }
        let piece = self.game.queue().nth_next_piece(3);
        let color = block_color(Some(piece));
        for offset in piece.coordinates() {
            draw_small_block(frame_buffer, [24, 13], Pos::new(1, 1) + offset, color);
        }

        if let Some(piece) = self.game.held_piece() {
            let color = block_color(Some(piece));
            for offset in piece.coordinates() {
                draw_big_block(frame_buffer, [22, 0], Pos::new(1, 1) + offset, color);
            }
        }

        let playfield = self.game.playfield();

        // Draw row clear animation
        if let Some((rows_cleared, frame)) = &self.row_clear_anim {
            let progress = *frame as f32 / ROW_CLEAR_TIME as f32;
            for x in 0..playfield.width() * 2 {
                const T1_START: f32 = 0.0;
                const T1_LEN: f32 = 0.0;
                const T1_END: f32 = T1_START + T1_LEN;
                const T2_START: f32 = T1_END;
                const T2_LEN: f32 = 1.0;
                const T2_END: f32 = T2_START + T2_LEN;
                const TOTAL_LEN: f32 = T2_END + 0.5;

                let t = progress * (1.0 + TOTAL_LEN) - x as f32 / (playfield.width() as f32 * 2.0);
                for y in rows_cleared {
                    for dy in 0..2 {
                        let h = frame_buffer.len();
                        let [r, g, b] =
                            &mut frame_buffer[h - 1 - (*y as usize * 2 + dy)][x as usize];
                        let t1 = (t - T1_START) / T1_LEN;
                        let t2 = (t - T2_START) / T2_LEN;

                        if t2 >= 0.0 {
                            let q = t2.clamp(0.0, 1.0);
                            *r = ((1.0 - q) * (*r as f32 / 2.0 + 0.5)).clamp(0.0, 255.0) as u8;
                            *g = ((1.0 - q) * (*g as f32 / 2.0 + 0.5)).clamp(0.0, 255.0) as u8;
                            *b = ((1.0 - q) * (*b as f32 / 2.0 + 0.5)).clamp(0.0, 255.0) as u8;
                        } else if t1 >= 0.0 {
                            // *r = (*r as f32 * (1.0 - t1) + t1 * 255.0).clamp(0.0, 255.0) as u8;
                            // *g = (*g as f32 * (1.0 - t1) + t1 * 255.0).clamp(0.0, 255.0) as u8;
                            // *b = (*b as f32 * (1.0 - t1) + t1 * 255.0).clamp(0.0, 255.0) as u8;
                        }
                    }
                }
            }
        }

        // Draw hard drop animation
        if let Some((end_piece, frame)) = &self.hard_drop_anim {
            const MAX_BRIGHTNESS: f32 = 0.5;
            const TINT: f32 = 1.0;

            let mut start_positions: Vec<Pos> = vec![];
            for pos in end_piece.coordinates() {
                if let Some(p) = start_positions.iter_mut().find(|p| p.x == pos.x) {
                    p.y = std::cmp::max(pos.y, p.y);
                } else {
                    start_positions.push(pos);
                }
            }

            for Pos { x, y: y0 } in start_positions {
                let y0 = (y0 + 1) * 2;
                let y1 = self.game.config().height as i8 * 2;
                for y in y0..y1 {
                    let y_frac = (y - y0) as f32 / (y1 - y0) as f32;
                    let t = (1.0 - *frame as f32 / HARD_DROP_TIME as f32 - y_frac * 2.0)
                        * MAX_BRIGHTNESS;
                    for dx in 0..2 {
                        let h = frame_buffer.len();
                        let [r, g, b] = &mut frame_buffer[h - 1 - y as usize][x as usize * 2 + dx];

                        let [r2, g2, b2] = block_color(Some(end_piece.piece));

                        *r = (*r as f32 * (1.0 - t)
                            + (r2 as f32 / 255.0 * TINT + (1.0 - TINT)) * (t * 255.0))
                            .clamp(0.0, 255.0) as u8;
                        *g = (*g as f32 * (1.0 - t)
                            + (g2 as f32 / 255.0 * TINT + (1.0 - TINT)) * (t * 255.0))
                            .clamp(0.0, 255.0) as u8;
                        *b = (*b as f32 * (1.0 - t)
                            + (b2 as f32 / 255.0 * TINT + (1.0 - TINT)) * (t * 255.0))
                            .clamp(0.0, 255.0) as u8;
                    }
                }
            }
        }
    }
}

fn draw_big_block(
    frame_buffer: &mut FrameBuffer,
    [base_x, base_y]: [usize; 2],
    pos: Pos,
    color: Rgb,
) {
    let h = frame_buffer.len();

    let fb_x = base_x + pos.x as usize * 2;
    let fb_y = base_y + pos.y as usize * 2;
    frame_buffer[h - 1 - fb_y][fb_x] = color;
    frame_buffer[h - 1 - fb_y][fb_x + 1] = color;
    frame_buffer[h - 1 - (fb_y + 1)][fb_x] = color;
    frame_buffer[h - 1 - (fb_y + 1)][fb_x + 1] = color;
}

fn draw_small_block(
    frame_buffer: &mut FrameBuffer,
    [base_x, base_y]: [usize; 2],
    pos: Pos,
    color: Rgb,
) {
    let h = frame_buffer.len();
    let fb_x = base_x + pos.x as usize;
    let fb_y = base_y + pos.y as usize;
    frame_buffer[h - 1 - fb_y][fb_x] = color;
}

fn block_color(piece: Option<Tetromino>) -> Rgb {
    match piece {
        None => BACKGROUND,
        Some(Tetromino::I) => rgb(0x00FFFF),
        Some(Tetromino::J) => rgb(0x0000FF),
        Some(Tetromino::L) => rgb(0xFF9900),
        Some(Tetromino::O) => rgb(0xFFFF00),
        Some(Tetromino::S) => rgb(0x00FF00),
        Some(Tetromino::T) => rgb(0xCC00FF),
        Some(Tetromino::Z) => rgb(0xFF0000),
    }
}

pub const fn rgb(hex: u32) -> Rgb {
    let r = (hex >> 16) as u8;
    let g = (hex >> 8) as u8;
    let b = hex as u8;
    [r, g, b]
}

pub const fn dim([r, g, b]: Rgb) -> Rgb {
    [r / 4, g / 4, b / 4]
}
