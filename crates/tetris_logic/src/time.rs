use std::fmt;
use std::ops::{Add, AddAssign};

pub trait GameTime:
    fmt::Debug + Copy + Ord + Add<Self::Duration, Output = Self> + AddAssign<Self::Duration>
{
    type Duration: fmt::Debug + Copy;
}

impl GameTime for u64 {
    type Duration = u64;
}

impl GameTime for std::time::Instant {
    type Duration = std::time::Duration;
}

#[cfg(feature = "web-time")]
impl GameTime for web_time::Instant {
    type Duration = web_time::Duration;
}
