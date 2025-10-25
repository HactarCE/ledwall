use std::time::Instant;

mod tetris;

pub type Rgb = [u8; 3];
pub type FrameBuffer = [[Rgb; WIDTH]; HEIGHT];

pub const BLACK: Rgb = [0_u8; 3];
pub const WHITE: Rgb = [255_u8; 3];

pub const FPS: usize = 60;
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
    pub r: bool,
    pub lt: bool,
    pub rt: bool,

    // Middle buttons
    pub plus: bool,
    pub minus: bool,
    pub star: bool,
    pub heart: bool,
}

pub struct App {
    start_time: Instant,
    frame_buffer: Box<FrameBuffer>,

    last_frame_time: std::time::Instant,

    tetris: tetris::Tetris,

    image_data: Option<Vec<u8>>,
}
impl Default for App {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
            frame_buffer: Box::new([[[0; 3]; WIDTH]; HEIGHT]),

            last_frame_time: Instant::now(),

            tetris: tetris::Tetris::default(),

            image_data: None,
        }
    }
}

impl App {
    pub fn buffer(&self) -> &FrameBuffer {
        &self.frame_buffer
    }
    pub fn flattened_buffer(&self) -> &[u8] {
        self.frame_buffer.as_flattened().as_flattened()
    }

    pub fn set_image(&mut self, image_data: Option<Vec<u8>>) {
        self.image_data = image_data;
    }

    pub fn update(&mut self, input: Input) {
        let now = Instant::now();
        self.last_frame_time = now;
        self.tetris.step(input);

        self.clear();
        self.display_rainbow();
        if let Some(img) = self.image_data.take() {
            self.display_image(&img);
            self.image_data = Some(img);
        } else {
            self.display_tetris();
            self.display_input(input);
        }
    }

    fn clear(&mut self) {
        self.frame_buffer.as_flattened_mut().fill(BLACK);
    }

    fn display_tetris(&mut self) {
        self.tetris.draw(&mut self.frame_buffer);
    }

    fn display_rainbow(&mut self) {
        let t = self.start_time.elapsed().as_secs_f64() / 2.0;
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let color = colorous::RAINBOW
                    .eval_continuous(((x as f64 + y as f64 * 2.0) / 64.0 - t).rem_euclid(1.0));
                self.frame_buffer[y][x] = color.as_array().map(|x| x / 3);
            }
        }
    }

    fn display_input(&mut self, input: Input) {
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

    fn display_image(&mut self, image_data: &[u8]) {
        for (&[r, g, b, a], out) in std::iter::zip(
            image_data.as_chunks().0,
            self.frame_buffer.as_flattened_mut(),
        ) {
            let a = a as f32 / 255.0;
            let [r2, g2, b2] = *out;
            *out = [(r, r2), (g, g2), (b, b2)]
                .map(|(x1, x2)| (x1 as f32 * a + x2 as f32 * (1.0 - a)) as u8);
        }
    }
}
