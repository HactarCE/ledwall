use tetris_logic::Pos;

use crate::{FrameBuffer, Rgb};

/// Tetris block coordinate transform to display on the screen.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Transform {
    /// Pixel coordinates of the top-left corner of the display.
    ///
    /// Note that display Y coordinates count down whereas Tetris Y coordinates
    /// count up, so this is close to the _minimum_ X coordinate and the
    /// _maximum_ Y coordinate.
    pub base: [i32; 2],

    /// Size of the display, in Tetris blocks.
    pub size: [i32; 2],

    /// Size of each Tetris block, in pixels.
    pub scale: usize,
}

impl Transform {
    pub const fn small(left_top: [i32; 2], size: [i32; 2]) -> Self {
        Self::new(left_top, size, 1)
    }
    pub const fn big(left_top: [i32; 2], size: [i32; 2]) -> Self {
        Self::new(left_top, size, 2)
    }
    pub const fn huge(left_top: [i32; 2], size: [i32; 2]) -> Self {
        Self::new(left_top, size, 3)
    }

    const fn new(left_top: [i32; 2], size: [i32; 2], scale: usize) -> Self {
        let [left, top] = left_top;
        Self {
            base: [left, top + size[1] * scale as i32],
            size,
            scale,
        }
    }

    pub fn base_pixel(self, pos: Pos) -> Option<[usize; 2]> {
        let x = pos.x as i32;
        let y = pos.y as i32;
        let [bx, by] = self.base;
        let [sx, sy] = self.size;
        ((0..sx).contains(&x) && (0..sy).contains(&y))
            .then(|| [x, y].map(|coordinate| coordinate * self.scale as i32))
            .map(|[x, y]| [bx + x, by - y - 1].map(|coordinate| coordinate as usize))
            .filter(|&xy| crate::xy_is_in_frame(xy))
    }

    pub fn pixels_of(self, pos: Pos) -> impl Iterator<Item = [usize; 2]> {
        let base_pixel = self.base_pixel(pos);
        (0..self.scale)
            .flat_map(move |dy| {
                (0..self.scale).filter_map(move |dx| base_pixel.map(|[x, y]| [x + dx, y - dy]))
            })
            .filter(|&xy| crate::xy_is_in_frame(xy))
    }

    pub fn border_pixels(self) -> impl Iterator<Item = [usize; 2]> {
        let [bx, by] = self.base;
        let [sx, sy] = self.size;
        let w = sx * self.scale as i32;
        let h = sy * self.scale as i32;
        let x2 = bx + w;
        let y1 = by - h - 1;

        let left = bx - 1;
        let top = y1;
        let right = x2;
        let bottom = by;
        Iterator::chain(
            (left..=right).flat_map(move |x| [[x, top], [x, bottom]]),
            (top + 1..=bottom - 1).flat_map(move |y| [[left, y], [right, y]]),
        )
        .filter_map(|[x, y]| Some([x.try_into().ok()?, y.try_into().ok()?]))
        .filter(|&xy| crate::xy_is_in_frame(xy))
    }

    pub fn fill_block(self, frame_buffer: &mut FrameBuffer, pos: Pos, color: Rgb) {
        for [fbx, fby] in self.pixels_of(pos) {
            frame_buffer[fby][fbx] = color;
        }
    }
    pub fn fill_border(self, frame_buffer: &mut FrameBuffer, color: Rgb) {
        for [fbx, fby] in self.border_pixels() {
            frame_buffer[fby][fbx] = color;
        }
    }
}
