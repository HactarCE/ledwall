use crate::{FrameBufferRect, Rgb, StaticImage};

pub type Font = fn(char) -> StaticImage;

pub fn width(s: &str, font: Font) -> usize {
    s.chars()
        .map(font)
        .map(|img| img.width() + 1)
        .sum::<usize>()
        - 1
}

pub fn draw(s: &str, font: Font, fb: &mut FrameBufferRect<'_>, tint: Rgb) {
    let mut x = 0;
    for img in s.chars().map(font) {
        img.draw_tinted(&mut fb.with_offset([x, 0]), tint);
        x += img.width() as isize + 1;
    }
}

pub const FONT_5PX: Font = |c| match c.to_ascii_lowercase() {
    ' ' => include_rgba_image!("font_5px/space.rgba"),
    '!' => include_rgba_image!("font_5px/bang.rgba"),
    '"' => include_rgba_image!("font_5px/quote.rgba"),
    '\'' => include_rgba_image!("font_5px/apostrophe.rgba"),
    '(' => include_rgba_image!("font_5px/lparen.rgba"),
    ')' => include_rgba_image!("font_5px/rparen.rgba"),
    '+' => include_rgba_image!("font_5px/plus.rgba"),
    '-' => include_rgba_image!("font_5px/hyphen.rgba"),
    '.' => include_rgba_image!("font_5px/period.rgba"),
    '/' => include_rgba_image!("font_5px/slash.rgba"),
    '0' => include_rgba_image!("font_5px/0.rgba"),
    '1' => include_rgba_image!("font_5px/1.rgba"),
    '2' => include_rgba_image!("font_5px/2.rgba"),
    '3' => include_rgba_image!("font_5px/3.rgba"),
    '4' => include_rgba_image!("font_5px/4.rgba"),
    '5' => include_rgba_image!("font_5px/5.rgba"),
    '6' => include_rgba_image!("font_5px/6.rgba"),
    '7' => include_rgba_image!("font_5px/7.rgba"),
    '8' => include_rgba_image!("font_5px/8.rgba"),
    '9' => include_rgba_image!("font_5px/9.rgba"),
    ':' => include_rgba_image!("font_5px/colon.rgba"),
    '<' => include_rgba_image!("font_5px/lt.rgba"),
    '=' => include_rgba_image!("font_5px/equals.rgba"),
    '>' => include_rgba_image!("font_5px/gt.rgba"),
    '?' => include_rgba_image!("font_5px/question.rgba"),
    '[' => include_rgba_image!("font_5px/lbracket.rgba"),
    ']' => include_rgba_image!("font_5px/rbracket.rgba"),
    '^' => include_rgba_image!("font_5px/caret.rgba"),
    '_' => include_rgba_image!("font_5px/underscore.rgba"),
    'a' => include_rgba_image!("font_5px/a.rgba"),
    'b' => include_rgba_image!("font_5px/b.rgba"),
    'c' => include_rgba_image!("font_5px/c.rgba"),
    'd' => include_rgba_image!("font_5px/d.rgba"),
    'e' => include_rgba_image!("font_5px/e.rgba"),
    'f' => include_rgba_image!("font_5px/f.rgba"),
    'g' => include_rgba_image!("font_5px/g.rgba"),
    'h' => include_rgba_image!("font_5px/h.rgba"),
    'i' => include_rgba_image!("font_5px/i.rgba"),
    'j' => include_rgba_image!("font_5px/j.rgba"),
    'k' => include_rgba_image!("font_5px/k.rgba"),
    'l' => include_rgba_image!("font_5px/l.rgba"),
    'm' => include_rgba_image!("font_5px/m.rgba"),
    'n' => include_rgba_image!("font_5px/n.rgba"),
    'o' => include_rgba_image!("font_5px/o.rgba"),
    'p' => include_rgba_image!("font_5px/p.rgba"),
    'q' => include_rgba_image!("font_5px/q.rgba"),
    'r' => include_rgba_image!("font_5px/r.rgba"),
    's' => include_rgba_image!("font_5px/s.rgba"),
    't' => include_rgba_image!("font_5px/t.rgba"),
    'u' => include_rgba_image!("font_5px/u.rgba"),
    'v' => include_rgba_image!("font_5px/v.rgba"),
    'w' => include_rgba_image!("font_5px/w.rgba"),
    'x' => include_rgba_image!("font_5px/x.rgba"),
    'y' => include_rgba_image!("font_5px/y.rgba"),
    'z' => include_rgba_image!("font_5px/z.rgba"),
    '{' => include_rgba_image!("font_5px/lbrace.rgba"),
    '}' => include_rgba_image!("font_5px/rbrace.rgba"),
    '\u{200A}' => StaticImage::EMPTY, // hair space
    _ => include_rgba_image!("font_5px/unknown.rgba"),
};

pub const FONT_10PX: Font = |c| match c {
    ' ' => include_rgba_image!("font_10px/space.rgba"),
    '0' => include_rgba_image!("font_10px/0.rgba"),
    '1' => include_rgba_image!("font_10px/1.rgba"),
    '2' => include_rgba_image!("font_10px/2.rgba"),
    '3' => include_rgba_image!("font_10px/3.rgba"),
    '4' => include_rgba_image!("font_10px/4.rgba"),
    '5' => include_rgba_image!("font_10px/5.rgba"),
    '6' => include_rgba_image!("font_10px/6.rgba"),
    '7' => include_rgba_image!("font_10px/7.rgba"),
    '8' => include_rgba_image!("font_10px/8.rgba"),
    '9' => include_rgba_image!("font_10px/9.rgba"),
    ':' => include_rgba_image!("font_10px/colon.rgba"),
    _ => include_rgba_image!("font_10px/unknown.rgba"),
};
