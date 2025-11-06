use rand::SeedableRng;
use tetris_logic::{FrameInput, Pos, Tetromino};

mod animations;
mod constants;
mod display;

use crate::{
    Activity, FrameBufferRect, FullInput, StaticImage, Widget, draw_opt_animation,
    step_opt_animation,
};
use animations::*;
use constants::{colors, coordinates};
use display::Transform;

pub struct Tetris {
    game: tetris_logic::Game<u64>,
    queue: [Option<Tetromino>; 4],

    soon_to_lock_anim: SoonToLockAnimation,
    locked_anim: Option<LockedAnimation>,
    hard_drop_anim: Option<HardDropAnimation>,
    clear_anim: Option<ClearAnimation>,

    big: bool,
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
            queue: [None; 4],

            soon_to_lock_anim: SoonToLockAnimation::default(),
            locked_anim: None,
            hard_drop_anim: None,
            clear_anim: None,

            big: false,
        }
    }
}

impl Activity for Tetris {
    fn reset(&mut self) {
        *self = Self::default();
    }

    fn menu_image(&self) -> StaticImage {
        include_rgba_image!("menu/tetris.rgba")
    }
}

impl Widget<FullInput> for Tetris {
    fn step(&mut self, input: FullInput) {
        let keys_down = input.any().current;
        let keys_pressed = input.any().pressed();

        self.big ^= keys_pressed.minus;

        self.soon_to_lock_anim.step();
        step_opt_animation(&mut self.locked_anim);
        step_opt_animation(&mut self.hard_drop_anim);
        step_opt_animation(&mut self.clear_anim);

        if self.clear_anim.is_some() {
            return; // freeze game
        }

        let result = self.game.step(
            1,
            FrameInput {
                left: keys_down.left,      // DAS
                right: keys_down.right,    // DAS
                soft_drop: keys_down.down, // DAS
                hard_drop: keys_pressed.up,
                rot_cw: keys_pressed.a || keys_pressed.y,
                rot_ccw: keys_pressed.b,
                rot_180: keys_pressed.x,
                hold: keys_pressed.l || keys_pressed.r || keys_pressed.lt || keys_pressed.rt,
            },
        );

        if let Ok(output) = &result {
            if let Some(rows_cleared) = &output.rows_cleared {
                self.clear_anim = Some(ClearAnimation::new(rows_cleared.clone()));
            }
            if let Some(locked_piece) = output.locked_piece {
                self.locked_anim = Some(LockedAnimation::new(locked_piece));
                self.soon_to_lock_anim.reset();
            }
            if let Some(Ok(dropped_piece)) = output.hard_drop
                && let Some(locked_piece) = output.locked_piece
            {
                let trail_len = dropped_piece.pos.y - locked_piece.pos.y;
                self.hard_drop_anim = Some(HardDropAnimation::new(trail_len, locked_piece));
            }
        }

        if self.game.can_soft_drop() {
            self.soon_to_lock_anim.reset();
        }

        let mut queue_iter = self.game.queue().next_pieces();
        self.queue.fill_with(|| queue_iter.next());
    }

    fn draw(&self, fb: &mut FrameBufferRect<'_>) {
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
            playfield.fill_block(fb, pos, self.soon_to_lock_anim.modify_color(falling_color));
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
            for (i, transform) in [
                coordinates::NEXT_PIECE,
                coordinates::NEXT_PIECE_2,
                coordinates::NEXT_PIECE_3,
                coordinates::NEXT_PIECE_4,
            ]
            .into_iter()
            .enumerate()
            {
                if let Some(piece) = self.queue[i] {
                    fill_piece_preview(transform, fb, piece);
                    transform.fill_border(fb, colors::NEXT_PIECE_BORDER);
                }
            }
        }

        // Draw locking animation
        draw_opt_animation(&self.locked_anim, fb, playfield);

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
