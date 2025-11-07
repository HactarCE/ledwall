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

    pub fn to_oklab(self) -> oklab::Oklab {
        oklab::srgb_to_oklab(self.0.into())
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

        let lab1 = self.to_oklab();
        let lab2 = other.to_oklab();

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

    /// Mixes multiple colors perceptually and uniformly using Oklab color
    /// space.
    pub fn mix_multiple(colors: impl IntoIterator<Item = Rgb>) -> Rgb {
        let [mut l, mut a, mut b] = [0.0, 0.0, 0.0];
        let mut count = 0.0;
        for color in colors {
            let lab = color.to_oklab();
            l += lab.l;
            a += lab.a;
            b += lab.b;
            count += 1.0;
        }

        Self(
            oklab::Oklab {
                l: l / count,
                a: a / count,
                b: b / count,
            }
            .to_srgb()
            .into(),
        )
    }

    pub fn lighten(self, t: f32) -> Rgb {
        self.mix(WHITE, t)
    }
    pub fn darken(self, t: f32) -> Rgb {
        self.mix(BLACK, t)
    }
}
