use chrono::{Datelike, Days, NaiveDate, Timelike, Weekday};

use crate::{Activity, FrameBufferRect, FullInput, Rgb, WHITE, Widget, text};

#[derive(Debug, Default)]
pub struct Clock;

impl Widget<FullInput> for Clock {
    fn draw(&self, fb: &mut FrameBufferRect<'_>) {
        let now = chrono::Local::now();
        let today = now.date_naive();
        let (pm, hour) = now.hour12();
        let minute = now.minute();
        crate::text::draw(
            &format!("{hour:>2}:{minute:02}"),
            crate::text::FONT_10PX,
            &mut fb.with_offset([1, 0]),
            WHITE,
        );
        crate::text::draw(
            if pm { "pm" } else { "am" },
            crate::text::FONT_5PX,
            &mut fb.with_offset([23, 12]),
            WHITE.darken(0.5),
        );
        // crate::text::draw(
        //     &today.format("%b %-d").to_string(),
        //     crate::text::FONT_5PX,
        //     &mut fb.with_offset([59, 10]),
        //     WHITE,
        // );

        // Use hair spaces to keep it compact
        let text = today.format("%b\u{200A}\u{200A}%-d").to_string();
        crate::text::draw(
            &text,
            crate::text::FONT_5PX,
            &mut fb.with_offset([32 - text::width(&text, crate::text::FONT_5PX) as isize, 59]),
            WHITE.darken(0.75),
        );

        let week_count = if today.leap_year()
            && NaiveDate::from_yo_opt(now.year(), 1).is_some_and(|d| d.weekday() == Weekday::Sun)
        {
            54
        } else {
            53
        };
        let mut year_calendar_fb = fb.with_offset([0, 64 - week_count]);

        let mut y = 0; // row (week)
        for ordinal in 1..=366 {
            let Some(date) = NaiveDate::from_yo_opt(now.year(), ordinal) else {
                continue;
            };

            let x = date.weekday().num_days_from_monday() as usize; // column (weekday)
            let color = if date
                    // .checked_sub_days(Days::new(time.second() as u64 % 7))
                    // .unwrap()
                    == today
            {
                [
                    Rgb::from_hex(0xFF0808), // Mon = red
                    Rgb::from_hex(0xCC5500), // Tue = orange
                    Rgb::from_hex(0xFFFF00), // Wed = yellow
                    Rgb::from_hex(0x00CC00), // Thu = green
                    Rgb::from_hex(0x0066FF), // Fri = blue
                    Rgb::from_hex(0x6600FF), // Sat = purple
                    Rgb::from_hex(0x9900CC), // Sun = magenta
                ][x]
            } else if date.month() == today.month() {
                let is_in_past = date < today;
                Rgb([60, 60, 100]).darken(if is_in_past { 0.5 } else { 0.0 })
            } else {
                Rgb([if date.month0() % 2 == 0 { 5 } else { 10 }; 3])
            };
            year_calendar_fb.set(x, y, color);
            if date.weekday() == Weekday::Sun {
                y += 1;
            }
        }
    }
}

impl Activity for Clock {
    fn menu_image(&self) -> crate::StaticImage {
        include_rgba_image!("activities/unknown.rgba")
    }

    fn reset(&mut self) {
        *self = Self::default()
    }

    fn stay_awake(&self) -> bool {
        true
    }
}
