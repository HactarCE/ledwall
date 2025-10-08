use rand::SeedableRng;
use tetris_logic::{Pos, Tetromino};

use crate::{FrameBuffer, Input, Rgb};

pub struct Tetris {
    game: tetris_logic::Game,
}

impl Default for Tetris {
    fn default() -> Self {
        Self {
            game: tetris_logic::Game::new(
                tetris_logic::Config::default(),
                Box::new(rand::rngs::SmallRng::from_os_rng()),
            ),
        }
    }
}

impl Tetris {
    pub fn step(&mut self, input: Input) {
        match (input.left, input.right) {
            (true, false) => _ = self.game.input_move_left(),
            (false, true) => _ = self.game.input_move_right(),
            _ => (),
        }
        if input.down {
            _ = self.game.input_soft_drop();
        }
        if input.up {
            _ = self.game.input_hard_drop();
        }
        if input.a {
            _ = self.game.input_rotate_cw();
        }
        if input.b {
            _ = self.game.input_rotate_ccw();
        }
        if input.x {
            _ = self.game.input_hold();
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
        None => rgb(0x333333),
        Some(Tetromino::I) => rgb(0x00FFFF),
        Some(Tetromino::J) => rgb(0x0000FF),
        Some(Tetromino::L) => rgb(0xFF9900),
        Some(Tetromino::O) => rgb(0xFFFF00),
        Some(Tetromino::S) => rgb(0x00FF00),
        Some(Tetromino::T) => rgb(0xCC00FF),
        Some(Tetromino::Z) => rgb(0xFF0000),
    }
}

pub fn rgb(hex: u32) -> Rgb {
    let r = (hex >> 16) as u8;
    let g = (hex >> 8) as u8;
    let b = hex as u8;
    [r, g, b]
}

pub fn dim([r, g, b]: Rgb) -> Rgb {
    [r / 2, g / 2, b / 2]
}
