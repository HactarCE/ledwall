use rand::SeedableRng;
use tetris_logic::{FrameInput, Pos, Tetromino};

mod animations;
mod constants;
mod display;

use crate::{
    Activity, FrameBufferRect, Input, StaticImage, Widget, draw_opt_animation, step_opt_animation,
};
use animations::*;
use constants::{colors, coordinates};
use display::Transform;

pub struct Tetris {
    game: tetris_logic::Game<u64>,

    clear_anim: Option<ClearAnimation>,
    hard_drop_anim: Option<HardDropAnimation>,
    lock_anim: Option<LockAnimation>,

    big: bool,
    last_input: Input,
}

impl Default for Tetris {
    fn default() -> Self {
        Self {
            game: tetris_logic::Game::new(
                tetris_logic::Config {
                    das: Some(constants::DAS),
                    ..Default::default()
                },
                0,
                Box::new(rand::rngs::SmallRng::from_os_rng()),
            ),

            clear_anim: None,
            hard_drop_anim: None,
            lock_anim: None,

            big: false,
            last_input: Input::default(),
        }
    }
}

impl Activity for Tetris {
    fn menu_image(&self) -> StaticImage {
        include_rgba_image!("menu/tetris.rgba")
    }
}

impl Widget<Input> for Tetris {
    fn step(&mut self, input: Input) {
        let last_input = std::mem::replace(&mut self.last_input, input);
        self.big ^= input.minus & !last_input.minus;

        step_opt_animation(&mut self.clear_anim);
        step_opt_animation(&mut self.hard_drop_anim);
        step_opt_animation(&mut self.lock_anim);

        if self.clear_anim.is_some() {
            return; // freeze game
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
                self.clear_anim = Some(ClearAnimation::new(rows_cleared.clone()));
            }
            if let Some(locked_piece) = output.locked_piece {
                self.lock_anim = Some(LockAnimation::new(locked_piece));
            }
            if let Some(Ok(dropped_piece)) = output.hard_drop
                && let Some(locked_piece) = output.locked_piece
            {
                let trail_len = dropped_piece.pos.y - locked_piece.pos.y;
                self.hard_drop_anim = Some(HardDropAnimation::new(trail_len, locked_piece));
            }
        }
    }

    fn draw(&mut self, fb: &mut FrameBufferRect<'_>) {
        let width = self.game.config().width;
        let height = self.game.config().height;

        let playfield = if self.big {
            coordinates::PLAYFIELD_3X
        } else {
            coordinates::PLAYFIELD
        };

        // Draw background
        fb.fill(colors::BACKGROUND);

        // Draw static blocks
        for y in 0..height as i8 {
            for x in 0..width as i8 {
                let pos = Pos { x, y };
                if let Some(piece) = self.game.playfield().get(pos).flatten() {
                    let color = colors::piece(piece).darken(colors::DARKEN_STATIC_BLOCKS);
                    playfield.fill_block(fb, pos, color);
                }
            }
        }

        // Draw border
        if !self.big {
            playfield.fill_border(fb, colors::PLAYFIELD_BORDER);
        }

        let falling_piece = self.game.falling_piece();
        let falling_color = colors::piece(falling_piece.piece);

        // Draw ghost
        if let Some(ghost_pos) = self.game.ghost_piece_pos() {
            let ghost_color = falling_color.darken(colors::DARKEN_GHOST);
            for pos in falling_piece
                .piece
                .coordinates_at(falling_piece.rot, ghost_pos)
            {
                playfield.fill_block(fb, pos, ghost_color);
            }
        }

        // Draw falling piece
        for pos in falling_piece.coordinates() {
            playfield.fill_block(fb, pos, falling_color);
        }

        if !self.big {
            // Draw held piece
            if let Some(piece) = self.game.held_piece() {
                let darken = if self.game.hold_used() {
                    colors::DARKEN_USED_HELD_PIECE
                } else {
                    0.0
                };
                fill_darkened_piece_preview(coordinates::HELD_PIECE, fb, piece, darken);
            }
            coordinates::HELD_PIECE.fill_border(fb, colors::HELD_PIECE_BORDER);

            // Draw next pieces
            let queue = self.game.queue();
            for (i, transform) in [
                coordinates::NEXT_PIECE,
                coordinates::NEXT_PIECE_2,
                coordinates::NEXT_PIECE_3,
                coordinates::NEXT_PIECE_4,
            ]
            .into_iter()
            .enumerate()
            {
                fill_piece_preview(transform, fb, queue.nth_next_piece(i));
                transform.fill_border(fb, colors::NEXT_PIECE_BORDER);
            }
        }

        // Draw locking animation
        draw_opt_animation(&self.lock_anim, fb, playfield);

        // Draw hard drop animation
        draw_opt_animation(&self.hard_drop_anim, fb, playfield);

        // Draw row clear animation
        draw_opt_animation(&self.clear_anim, fb, playfield);
    }
}

fn fill_piece_preview(transform: Transform, fb: &mut FrameBufferRect<'_>, piece: Tetromino) {
    fill_darkened_piece_preview(transform, fb, piece, 0.0);
}

fn fill_darkened_piece_preview(
    transform: Transform,
    fb: &mut FrameBufferRect<'_>,
    piece: Tetromino,
    darken: f32,
) {
    let color = colors::piece(piece)
        .darken(colors::DARKEN_STATIC_BLOCKS)
        .darken(darken);
    for offset in piece.coordinates() {
        transform.fill_block(fb, Pos::new(1, 1) + offset, color);
    }
}
