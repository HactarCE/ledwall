use std::sync::LazyLock;

use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveTime, TimeDelta, Timelike, Weekday};

use crate::calendar::CalendarEventStatus;
use crate::widgets::{WebLoader, WebLoaderConfig};
use crate::{
    Activity, FONT_5PX, FONT_10PX, FrameBufferRect, FullInput, Rgb, Text, Tint, TintFn, WHITE,
    Widget,
};

const CALENDAR_IDS: &str = include_str!("../../../../calendars.txt");

const CALENDAR_START_HOUR: u32 = 6;
const CALENDAR_CHUNK_DURATION: TimeDelta = TimeDelta::minutes(30);
const CALENDAR_CHUNK_COUNT: usize = 18 * 2;
static TIME_CHUNKS: LazyLock<Vec<[NaiveTime; 2]>> = LazyLock::new(|| {
    let start_time = NaiveTime::from_hms_opt(CALENDAR_START_HOUR, 0, 0).unwrap();
    (0..CALENDAR_CHUNK_COUNT)
        .map(|i| {
            let t1 = start_time + CALENDAR_CHUNK_DURATION * i as i32;
            let t2 = t1 + CALENDAR_CHUNK_DURATION;
            [t1, t2]
        })
        .collect()
});

#[derive(Debug)]
struct CalendarTaskOutput {
    date: NaiveDate,
    calendars: Vec<CalendarData>,
}

#[derive(Debug)]
struct CalendarData {
    color: Rgb,
    events: crate::calendar::CalendarEvents,
}

#[derive(Debug)]
pub struct Clock {
    now: DateTime<Local>,
    calendar_task: WebLoader<CalendarTaskOutput>,
}

impl Default for Clock {
    fn default() -> Self {
        Self {
            now: Local::now(),
            calendar_task: WebLoader::new(WebLoaderConfig::default(), fetch_calendar_events),
        }
    }
}

fn day_bounds(date: NaiveDate) -> Option<[DateTime<Local>; 2]> {
    let start_of_day = date
        .and_time(NaiveTime::default())
        .and_local_timezone(Local)
        .single()?;
    let end_of_day = start_of_day
        .with_time(NaiveTime::from_hms_opt(23, 59, 59)?)
        .single()?;
    Some([start_of_day, end_of_day])
}

impl Widget<FullInput> for Clock {
    fn step(&mut self, _input: FullInput) {
        self.now = Local::now();

        if let Some(task_output) = self.calendar_task.get()
            && task_output.date != self.now.date_naive()
        {
            self.calendar_task.invalidate();
        }
        self.calendar_task.step(());
    }

    fn draw(&self, fb: &mut FrameBufferRect<'_>) {
        let now = self.now;
        let today = now.date_naive();
        let (pm, hour) = now.hour12();
        let minute = now.minute();

        // Draw hour:minute and AM/PM
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
            format!("{hour:>2}\u{200A}:\u{200A}{minute:02}"),
            FONT_10PX,
            soft_rainbow.at([0, 0]),
        )
        .draw(&mut fb.with_offset([0, 0]));
        Text::new(
            if pm { "pm" } else { "am" },
            FONT_5PX,
            soft_rainbow.at([23, 12]),
        )
        .draw(&mut fb.with_offset([23, 12]));

        // Draw month/day, using hair spaces for legibility
        let text = Text::new(
            today.format("%-m\u{200A}/\u{200A}%-d").to_string(),
            FONT_5PX,
            WHITE.darken(0.25),
        );
        text.draw(&mut fb.with_offset([32 - text.width() as isize, 59]));

        // Draw year calendar
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
            } else if (date.month0() ^ today.month0()).is_multiple_of(2) {
                Rgb([70; 3])
            } else {
                Rgb([20; 3])
            };
            year_calendar_fb.set(x, y, color);
            if date.weekday() == last_day_of_week {
                y += 1;
            }
        }

        // Draw calendar events for the day
        let calendar_count = CALENDAR_IDS.lines().filter(|l| !l.is_empty()).count();
        let x: isize = 28;
        let y: isize = 20;
        let w: usize = calendar_count;
        let h: usize = CALENDAR_CHUNK_COUNT;
        self.calendar_task
            .draw(&mut fb.rect([x - 1, y - 1], [w + 2, h + 2]));
        if let Some(task_output) = self.calendar_task.get() {
            // Draw brackets
            let c = WHITE.darken(0.5);
            fb.rect([x - 1, y - 1], [w + 2, 1]).fill(c);
            fb.rect([x - 1, y + h as isize], [w + 2, 1]).fill(c);
            fb.set(x as usize - 1, y as usize, c);
            fb.set(x as usize + w, y as usize, c);
            fb.set(x as usize - 1, y as usize + h - 1, c);
            fb.set(x as usize + w, y as usize + h - 1, c);

            let mut draw_time_indicator = |time: NaiveTime, ends_color: Rgb, fill_color: Rgb| {
                if let Some(i) = TIME_CHUNKS
                    .iter()
                    .position(|&[lo, hi]| lo <= time && time < hi)
                {
                    let mut row = fb.rect([x - 1, y + i as isize], [w + 2, 1]);
                    row.fill(ends_color);
                    row.rect([1, 0], [w, 1]).fill(fill_color);
                }
            };

            // Draw indicators
            for (hour, color) in [(12, Rgb::from_hex(0x00CCFF)), (18, Rgb::from_hex(0xFF9900))] {
                draw_time_indicator(
                    NaiveTime::from_hms_opt(hour, 0, 0).unwrap(),
                    color.darken(0.7),
                    color.darken(0.8),
                );
            }
            draw_time_indicator(now.time(), WHITE, WHITE.darken(0.75));

            // Draw events
            let Some([start_of_day, end_of_day]) = day_bounds(task_output.date) else {
                return;
            };

            for (dx, calendar) in task_output.calendars.iter().enumerate() {
                for event in &calendar.events.items {
                    if event.status == CalendarEventStatus::Confirmed
                        && let Some(start) = event.start.and_then(|t| t.date_time)
                        && let Some(end) = event.end.and_then(|t| t.date_time)
                        && end > start_of_day
                        && start < end_of_day
                    {
                        let start = std::cmp::max(start, start_of_day).time();
                        let end = std::cmp::min(end, end_of_day).time();
                        for (dy, &[t1, t2]) in TIME_CHUNKS.iter().enumerate() {
                            if start < t2 && end > t1 {
                                fb.set(x as usize + dx, y as usize + dy, calendar.color);
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Activity for Clock {
    fn menu_image(&self) -> crate::StaticImage {
        include_rgba_image!("activities/clock.rgba")
    }

    fn reset(&mut self) {
        *self = Self::default();
    }

    fn stay_awake(&self) -> bool {
        true
    }
}

/// Fetches calendar events, or returns `None` in case of any error.
///
/// Blocks until an access token is acquired.
fn fetch_calendar_events() -> Option<CalendarTaskOutput> {
    crate::calendar::wait_for_access_token();
    let now = Local::now();
    let [start_of_day, end_of_day] = day_bounds(now.date_naive())?;
    Some(CalendarTaskOutput {
        date: now.date_naive(),
        calendars: CALENDAR_IDS
            .lines()
            .filter(|l| !l.is_empty())
            .filter_map(|line| {
                let (color_str, calendar_id) = line.split_once(" ")?;
                let color = Rgb::from_hex(u32::from_str_radix(color_str, 16).unwrap_or(0xFFFFFF));
                let events = crate::calendar::get_events(calendar_id, start_of_day, end_of_day)?;
                Some(CalendarData { color, events })
            })
            .collect(),
    })
}
