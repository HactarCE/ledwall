use std::time::Instant;

pub const FPS: usize = 180;
pub const WIDTH: usize = 32;
pub const HEIGHT: usize = 64;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Input {
    // D pad
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,

    // Thumb buttons
    pub a: bool,
    pub b: bool,
    pub x: bool,
    pub y: bool,

    // Shoulder buttons
    pub l: bool,
    pub l2: bool,
    pub r: bool,
    pub r2: bool,

    // Middle buttons
    pub plus: bool,
    pub minus: bool,
    pub star: bool,
    pub heart: bool,
}

#[derive(Debug)]
pub struct App {
    start_time: Instant,
    frame_buffer: Box<[[[u8; 3]; WIDTH]; HEIGHT]>,
}
impl Default for App {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
            frame_buffer: Box::new([[[0; 3]; WIDTH]; HEIGHT]),
        }
    }
}

impl App {
    pub fn buffer(&self) -> &[[[u8; 3]; WIDTH]; HEIGHT] {
        &self.frame_buffer
    }
    pub fn flattened_buffer(&self) -> &[u8] {
        self.frame_buffer.as_flattened().as_flattened()
    }

    pub fn update(&mut self, input: Input) {
        let t = self.start_time.elapsed().as_secs_f64() / 2.0;
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let color = colorous::RAINBOW
                    .eval_continuous(((x as f64 + y as f64 * 2.0) / 64.0 - t).rem_euclid(1.0));
                self.frame_buffer[y][x] = color.as_array();
            }
        }
        for (i, bit) in [
            input.up,
            input.down,
            input.left,
            input.right,
            input.a,
            input.b,
            input.x,
            input.y,
        ]
        .into_iter()
        .enumerate()
        {
            self.frame_buffer[0][i] = [if bit { 255 } else { 0 }; 3];
        }
    }
}
