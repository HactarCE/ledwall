use crate::{FrameBufferRect, Rgb};

macro_rules! include_rgba_image {
    ($path:literal) => {
        $crate::image::StaticImage(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../img/rgba/",
            $path,
        )))
    };
}

/// Static RGBA image.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct StaticImage(pub &'static [u8]);

impl StaticImage {
    pub const EMPTY: Self = Self(&[0_u8; 8]);

    pub fn width(self) -> usize {
        u32::from_le_bytes([0, 1, 2, 3].map(|i| self.0[i])) as usize
    }
    pub fn height(self) -> usize {
        u32::from_le_bytes([4, 5, 6, 7].map(|i| self.0[i])) as usize
    }
    pub fn size(self) -> [usize; 2] {
        [self.width(), self.height()]
    }
    pub fn rgba_pixels(self) -> &'static [[u8; 4]] {
        bytemuck::cast_slice(&self.0[8..])
    }

    pub fn draw(self, fb: &mut FrameBufferRect<'_>) {
        self.draw_with_color_fn(fb, |c| c);
    }

    pub fn draw_with_color_fn(
        self,
        fb: &mut FrameBufferRect<'_>,
        mut color_fn: impl FnMut(Rgb) -> Rgb,
    ) {
        self.draw_with_custom_blend(fb, |fb_pixel, image_pixel, alpha| {
            fb_pixel.mix(color_fn(image_pixel), alpha as f32 / 255.0)
        });
    }

    pub fn draw_tinted(self, fb: &mut FrameBufferRect<'_>, tint: Rgb) {
        self.draw_with_color_fn(fb, |_| tint);
    }

    /// Draws an image to the screen using a custom blend function, given the
    /// screen pixel, image pixel, and image alpha.
    pub fn draw_with_custom_blend(
        self,
        fb: &mut FrameBufferRect<'_>,
        mut blend: impl FnMut(Rgb, Rgb, u8) -> Rgb,
    ) {
        let w = self.width();
        let h = self.height();
        let image_pixels = self.rgba_pixels();
        assert_eq!(
            image_pixels.len(),
            w * h,
            "image data len does not match width*height"
        );

        let mut i = 0;
        for y in 0..self.height() {
            for x in 0..self.width() {
                if let Some(fb_pixel) = fb.get_mut(x, y) {
                    let [r, g, b, a] = image_pixels[i];
                    *fb_pixel = blend(*fb_pixel, Rgb([r, g, b]), a);
                }
                i += 1;
            }
        }
    }
}
