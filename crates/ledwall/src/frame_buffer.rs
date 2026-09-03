use std::fmt;
use std::ops::{Index, IndexMut};

use crate::{HEIGHT, Rgb, Tint, WIDTH};

pub type FrameBuffer = [[Rgb; WIDTH]; HEIGHT];

/// Rectangular region within the frame buffer.
pub struct FrameBufferRect<'a> {
    frame_buffer: &'a mut FrameBuffer,
    offset: [isize; 2],
    size: [usize; 2],
}

impl fmt::Debug for FrameBufferRect<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FrameBufferRect")
            .field("offset", &self.offset)
            .field("size", &self.size)
            .finish()
    }
}

impl<'a> FrameBufferRect<'a> {
    pub fn new(frame_buffer: &'a mut FrameBuffer) -> FrameBufferRect<'a> {
        let size = [frame_buffer[0].len(), frame_buffer.len()];
        FrameBufferRect {
            frame_buffer,
            offset: [0; 2],
            size,
        }
    }

    pub fn with_offset<'b>(&'b mut self, [dx, dy]: [isize; 2]) -> FrameBufferRect<'b> {
        let [x, y] = self.offset;
        let [w, h] = self.size;

        FrameBufferRect {
            frame_buffer: self.frame_buffer,
            offset: [x + dx, y + dy],
            size: [
                (w as isize - dx).max(0) as usize,
                (h as isize - dy).max(0) as usize,
            ],
        }
    }
    pub fn with_size<'b>(&'b mut self, [width, height]: [usize; 2]) -> FrameBufferRect<'b> {
        let [w, h] = self.size;
        FrameBufferRect {
            frame_buffer: self.frame_buffer,
            offset: self.offset,
            size: [width.min(w), height.min(h)],
        }
    }
    /// Returns a rectangular subregion, given the coordinates of the top left
    /// and the size.
    pub fn rect<'b>(
        &'b mut self,
        [dx, dy]: [isize; 2],
        [width, height]: [usize; 2],
    ) -> FrameBufferRect<'b> {
        let [x, y] = self.offset;
        let [w, h] = self.size;

        FrameBufferRect {
            frame_buffer: self.frame_buffer,
            offset: [x + dx, y + dy],
            size: [
                width.min((w as isize - dx).max(0) as usize),
                height.min((h as isize - dy).max(0) as usize),
            ],
        }
    }
    /// Returns a centered rectangular subregion.
    pub fn centered_rect<'b>(&'b mut self, [width, height]: [usize; 2]) -> FrameBufferRect<'b> {
        let [w, h] = self.size;
        let dx = w.saturating_sub(width) as isize / 2;
        let dy = h.saturating_sub(height) as isize / 2;
        self.rect([dx, dy], [width, height])
    }
    /// Returns the largest centered square subregion.
    pub fn centered_square<'b>(&'b mut self) -> FrameBufferRect<'b> {
        let [w, h] = self.size;
        self.centered_rect([std::cmp::min(w, h); 2])
    }

    pub fn width(&self) -> usize {
        self.size[0]
    }
    pub fn height(&self) -> usize {
        self.size[1]
    }
    pub fn size(&self) -> [usize; 2] {
        self.size
    }

    pub fn fill(&mut self, tint: impl Tint) {
        let x0 = (-self.offset[0]).max(0) as usize;
        let y0 = (-self.offset[1]).max(0) as usize;
        for (y, row) in self.rows().enumerate() {
            for (x, pixel) in row.iter_mut().enumerate() {
                *pixel = tint.eval_tint([x0 + x, y0 + y], *pixel);
            }
        }
    }
    pub fn rows(&mut self) -> impl Iterator<Item = &mut [Rgb]> {
        let max_w = self.frame_buffer[0].len() as isize;
        let max_h = self.frame_buffer.len() as isize;
        let [bx, by] = self.offset;
        let [w, h] = self.size;

        self.frame_buffer[by.clamp(0, max_h) as usize..][..h]
            .iter_mut()
            .map(move |row| &mut row[bx.clamp(0, max_w) as usize..][..w])
    }
    pub fn pixels(&mut self) -> impl Iterator<Item = &mut Rgb> {
        self.rows().flatten()
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut Rgb> {
        let [w, h] = self.size;
        if !(x < w && y < h) {
            return None;
        }
        let [bx, by] = self.offset;
        self.frame_buffer
            .get_mut(usize::try_from(by + y as isize).ok()?)?
            .get_mut(usize::try_from(bx + x as isize).ok()?)
    }

    pub fn set(&mut self, x: usize, y: usize, tint: impl Tint) {
        if let Some(out) = self.get_mut(x, y) {
            *out = tint.eval_tint([x, y], *out);
        }
    }
}

impl Index<[usize; 2]> for FrameBufferRect<'_> {
    type Output = Rgb;

    fn index(&self, [x, y]: [usize; 2]) -> &Self::Output {
        let [w, h] = self.size;
        assert!(x < w && y < h, "coordinates out of bounds");
        let [bx, by] = self.offset;
        &self.frame_buffer[(by + y as isize) as usize][(bx + x as isize) as usize]
    }
}

impl IndexMut<[usize; 2]> for FrameBufferRect<'_> {
    fn index_mut(&mut self, [x, y]: [usize; 2]) -> &mut Self::Output {
        let [w, h] = self.size;
        assert!(x < w && y < h, "coordinates out of bounds");
        let [bx, by] = self.offset;
        &mut self.frame_buffer[(by + y as isize) as usize][(bx + x as isize) as usize]
    }
}
