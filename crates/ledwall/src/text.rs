use crate::{FrameBufferRect, Rgb, StaticImage};

pub fn layout(s: &str) -> impl Iterator<Item = StaticImage> {
    s.chars().map(|c| match c {
        '0' => include_rgba_image!("font/0.rgba"),
        '1' => include_rgba_image!("font/1.rgba"),
        '2' => include_rgba_image!("font/2.rgba"),
        '3' => include_rgba_image!("font/3.rgba"),
        '4' => include_rgba_image!("font/4.rgba"),
        '5' => include_rgba_image!("font/5.rgba"),
        '6' => include_rgba_image!("font/6.rgba"),
        '7' => include_rgba_image!("font/7.rgba"),
        '8' => include_rgba_image!("font/8.rgba"),
        '9' => include_rgba_image!("font/9.rgba"),
        ':' => include_rgba_image!("font/colon.rgba"),
        '.' => include_rgba_image!("font/period.rgba"),
        _ => include_rgba_image!("font/unknown.rgba"),
    })
}

pub fn width(s: &str) -> usize {
    layout(s).map(|img| img.width() + 1).sum::<usize>() - 1
}

pub fn draw(s: &str, fb: &mut FrameBufferRect<'_>, tint: Rgb) {
    let mut x = 0;
    for img in layout(s) {
        img.draw_tinted(&mut fb.with_offset([x, 0]), tint);
        x += img.width() as isize + 1;
    }
}
