use chrono::{Datelike, NaiveDate, Timelike, Weekday};

use crate::{
    Activity, FONT_5PX, FONT_10PX, FrameBufferRect, FullInput, Rgb, Text, Tint, TintFn, WHITE,
    Widget,
};

#[derive(Debug, Default)]
pub struct Clock;

impl Widget<FullInput> for Clock {
    fn draw(&self, fb: &mut FrameBufferRect<'_>) {
        let now = chrono::Local::now();
        let today = now.date_naive();
        let (pm, hour) = now.hour12();
        let minute = now.minute();

        let millis_per_min = 1000 * 60;
        let fraction_of_minute =
            (now.timestamp_millis() % millis_per_min) as f64 / millis_per_min as f64;
        let soft_rainbow = TintFn(|[x, y], _| {
            colorous::RAINBOW
                .eval_continuous(
                    (fraction_of_minute - x as f64 / 64.0 - y as f64 / 128.0).rem_euclid(1.0),
                )
                .into()
        });

        Text::new(
            format!("{hour:>2}:{minute:02}"),
            FONT_10PX,
            soft_rainbow.at([1, 0]),
        )
        .draw(&mut fb.with_offset([1, 0]));
        Text::new(
            if pm { "pm" } else { "am" },
            FONT_5PX,
            soft_rainbow.at([22, 12]),
        )
        .draw(&mut fb.with_offset([22, 12]));

        // Use hair spaces to keep it compact
        let text = Text::new(
            today.format("%-m\u{200A}/\u{200A}%-d").to_string(),
            FONT_5PX,
            WHITE,
        );
        text.draw(&mut fb.with_offset([31 - text.width() as isize, 59]));

        let last_day_of_week = Weekday::Sat;

        let week_count = if today.leap_year()
            && NaiveDate::from_yo_opt(now.year(), 1)
                .is_some_and(|d| d.weekday() == last_day_of_week)
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

            let x = date.weekday().num_days_from_sunday() as usize; // column (weekday)
            let color = if date == today {
                [
                    Rgb::from_hex(0x9900CC), // Sun = magenta
                    Rgb::from_hex(0xFF0808), // Mon = red
                    Rgb::from_hex(0xFF6600), // Tue = orange
                    Rgb::from_hex(0xFFFF00), // Wed = yellow
                    Rgb::from_hex(0x00CC00), // Thu = green
                    Rgb::from_hex(0x0066FF), // Fri = blue
                    Rgb::from_hex(0x6600FF), // Sat = purple
                ][x]
            } else if date.month() == today.month() {
                Rgb([100; 3])
            } else if (date.month0() ^ today.month0()) % 2 == 0 {
                Rgb([70; 3])
            } else {
                Rgb([20; 3])
            };
            year_calendar_fb.set(x, y, color);
            if date.weekday() == last_day_of_week {
                y += 1;
            }
        }
    }
}

impl Activity for Clock {
    fn menu_image(&self) -> crate::StaticImage {
        include_rgba_image!("activities/clock.rgba")
    }

    fn reset(&mut self) {
        *self = Self::default()
    }

    fn stay_awake(&self) -> bool {
        true
    }
}
