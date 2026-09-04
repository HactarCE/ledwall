use std::{collections::HashMap, fmt, sync::LazyLock};

use crate::StaticImage;

macro_rules! font_definition {
    ($static_var_name:ident = $filename:literal) => {
        pub static $static_var_name: LazyLock<Font> = LazyLock::new(|| {
            Font::load(include_str!(concat!(
                "../../../img/font/",
                $filename,
                ".txt"
            )))
            .unwrap_or_else(|e| {
                eprintln!("error loading font {}: {}", $filename, e);
                Font::default()
            })
        });
    };
}

font_definition!(FONT_MONO_DIGITS_5PX = "mono_digits_5px");
font_definition!(FONT_PROP_5PX = "prop_5px");
font_definition!(FONT_MONO_DIGITS_10PX = "mono_digits_10px");

#[derive(Debug, Default, Clone)]
pub struct Font {
    default: StaticImage,
    chars: HashMap<char, StaticImage>,
}

impl Font {
    /// Loads a font from a string.
    pub fn load(s: &str) -> Result<Font, FontLoadError> {
        let mut chars = HashMap::<char, Vec<Vec<bool>>>::new();
        let mut case_fold = false;

        let mut current_chars: Vec<char> = vec![];
        for (i, mut line) in s.lines().enumerate() {
            line = match line.split_once("//") {
                Some((before_comment, _after_comment)) => before_comment,
                None => line,
            }
            .trim();

            if line.is_empty() {
                continue;
            } else if line == "case_fold" {
                case_fold = true;
            } else if line == "default" {
                current_chars = vec!['\0'];
            } else if let Some(rest) = line.strip_prefix("c ") {
                current_chars = rest
                    .trim()
                    .split_whitespace()
                    .map(|s| exactly_one_char(i, s))
                    .collect::<Result<Vec<char>, FontLoadError>>()?;
                if current_chars.is_empty() {
                    return Err(FontLoadError::new(
                        i,
                        "expected at least one char after 'c' directive",
                    ));
                }
                for &c in &current_chars {
                    chars.entry(c).or_default();
                }
            } else if let Some(rest) = line.strip_prefix("u ") {
                let codepoint =
                    u32::from_str_radix(rest.trim(), 16).map_err(|e| FontLoadError::new(i, e))?;
                current_chars = vec![
                    char::from_u32(codepoint)
                        .ok_or_else(|| FontLoadError::new(i, "invalid unicode char"))?,
                ];
                for &c in &current_chars {
                    chars.entry(c).or_default();
                }
            } else {
                if current_chars.is_empty() {
                    return Err(FontLoadError::new(
                        i,
                        "missing 'default', 'c', or 'u' directive",
                    ));
                }
                let sections: Vec<&str> = line.split("   ").collect();
                if current_chars.len() != sections.len() {
                    return Err(FontLoadError::new(
                        i,
                        format!(
                            "expected {} sections; got {}",
                            current_chars.len(),
                            sections.len(),
                        ),
                    ));
                }
                for (&c, row) in current_chars.iter().zip(sections) {
                    chars.entry(c).or_default().push(
                        row.chars()
                            .filter(|&c| c != ' ')
                            .map(|c| match c {
                                '#' => Ok(true),
                                '.' => Ok(false),
                                _ => {
                                    Err(FontLoadError::new(i, format!("unknown bitmap char {c:?}")))
                                }
                            })
                            .collect::<Result<Vec<bool>, FontLoadError>>()?,
                    )
                }
            }
        }

        let mut char_img_data: HashMap<char, Vec<u8>> = chars
            .into_iter()
            .map(|(c, img_rows)| {
                let h = img_rows.len();
                let w = img_rows.iter().map(|row| row.len()).max().unwrap_or(0);
                let mut bytes = vec![];
                bytes.extend_from_slice(&(w as u32).to_ne_bytes());
                bytes.extend_from_slice(&(h as u32).to_ne_bytes());
                for row in img_rows {
                    for i in 0..w {
                        bytes.extend_from_slice(match row.get(i) {
                            Some(true) => &[255; 4],
                            _ => &[0; 4],
                        });
                    }
                }
                (c, bytes)
            })
            .collect();

        let default = char_img_data
            .remove(&'\0')
            .ok_or_else(|| FontLoadError::without_line_number("missing 'default' char"))?;

        let mut chars: HashMap<char, StaticImage> = char_img_data
            .into_iter()
            .map(|(c, img_data)| (c, StaticImage(img_data.leak())))
            .collect();

        if case_fold {
            let mut new_chars = HashMap::new();
            for (c, img) in chars {
                new_chars.insert(c.to_ascii_lowercase(), img);
                new_chars.insert(c.to_ascii_uppercase(), img);
            }
            chars = new_chars;
        }

        Ok(Font {
            default: StaticImage(default.leak()),
            chars,
        })
    }

    pub fn get(&self, c: char) -> StaticImage {
        *self.chars.get(&c).unwrap_or(&self.default)
    }
}

#[derive(Debug)]
pub struct FontLoadError {
    msg: String,
    line_number: Option<usize>,
}

impl std::error::Error for FontLoadError {}

impl fmt::Display for FontLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { msg, line_number } = self;
        write!(f, "{msg}")?;
        if let Some(l) = line_number {
            write!(f, " on line {l}")?;
        }
        Ok(())
    }
}

impl FontLoadError {
    pub fn new(line_index: usize, msg: impl ToString) -> Self {
        Self {
            msg: msg.to_string(),
            line_number: Some(line_index + 1),
        }
    }

    pub fn without_line_number(msg: impl ToString) -> Self {
        Self {
            msg: msg.to_string(),
            line_number: None,
        }
    }
}

fn exactly_one_char(line_index: usize, s: &str) -> Result<char, FontLoadError> {
    let mut chars = s.chars();
    let ret = chars
        .next()
        .ok_or_else(|| FontLoadError::new(line_index, "expected char; got nothing"))?;
    if chars.next().is_some() {
        return Err(FontLoadError::new(
            line_index,
            format!("expected one char; got {s:?}"),
        ));
    }
    Ok(ret)
}
