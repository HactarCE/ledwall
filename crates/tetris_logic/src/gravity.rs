use crate::{Game, GameTime};

pub type SpeedCurve<Time: GameTime> = Box<dyn Fn(&Game) -> Time::Duration>;

/// Returns the [default speed curve](https://tetris.wiki/Marathon) for Marathon
/// mode based on Tetris Worlds.
///
/// If `master_mode` is `true`, then gravity continues increasing after level
/// 15. If `master_mode` is `false`, then gravity is capped at 20G.
pub fn default_speed_curve<Time:GameTime>(master_mode: bool)
