use crate::mix;

pub const BLACK: Rgb = Rgb::from_hex(0x000000);
pub const WHITE: Rgb = Rgb::from_hex(0xFFFFFF);

/// sRGB color 0-255
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(transparent)]
pub struct Rgb(pub [u8; 3]);

impl Rgb {
    pub const fn from_hex(hex: u32) -> Rgb {
        let r = (hex >> 16) as u8;
        let g = (hex >> 8) as u8;
        let b = hex as u8;
        Rgb([r, g, b])
    }

    /// Mixes two colors perceptually using Oklab color space.
    ///
    /// `t` is clamped between `0.0` and `1.0`.
    pub fn mix(self, other: Rgb, t: f32) -> Rgb {
        // optimize for special cases
        if t <= 0.0 {
            return self;
        }
        if t >= 1.0 {
            return other;
        }

        let lab1 = oklab::srgb_to_oklab(self.0.into());
        let lab2 = oklab::srgb_to_oklab(other.0.into());

        Self(
            oklab::Oklab {
                l: mix(lab1.l..lab2.l, t),
                a: mix(lab1.a..lab2.a, t),
                b: mix(lab1.b..lab2.b, t),
            }
            .to_srgb()
            .into(),
        )
    }
    // /// Mixes between `self` and `other` in an approximately perceptually
    // /// uniform way, assuming `other` is much darker than `self`.
    // pub fn mix_to_dark(self, other: Rgb, t: f32) -> Rgb {}

    pub fn lighten(self, t: f32) -> Rgb {
        self.mix(WHITE, t)
    }
    pub fn darken(self, t: f32) -> Rgb {
        self.mix(BLACK, t)
    }
}
