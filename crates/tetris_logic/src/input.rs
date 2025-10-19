use crate::GameTime;
use crate::config::Das;

/// Inputs for a frame.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FrameInput {
    /// Attempt to move the falling piece to the left (auto-repeats).
    pub left: bool,
    /// Attempt to move the falling piece to the right (auto-repeats).
    pub right: bool,
    /// Attempt to soft-drop the falling piece and (auto-repeats).
    pub soft_drop: bool,
    /// Hard-drop the falling piece.
    pub hard_drop: bool,
    /// Attempt to rotate the piece 90° clockwise.
    pub rot_cw: bool,
    /// Attempt to rotate the piece 90° counterclockwise.
    pub rot_ccw: bool,
    /// Attempt to rotate the piece 180°.
    pub rot_180: bool,
    /// Attempt to swaps the falling piece with the held piece.
    pub hold: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputState<Time: GameTime> {
    pub left: DasState<Time>,
    pub right: DasState<Time>,
    pub soft_drop: DasState<Time>,
    pub keys_down: FrameInput,
}
impl<Time: GameTime> Default for InputState<Time> {
    fn default() -> Self {
        Self {
            left: DasState::Released,
            right: DasState::Released,
            soft_drop: DasState::Released,
            keys_down: FrameInput::default(),
        }
    }
}
impl<Time: GameTime> InputState<Time> {
    /// Updates the input state and returns which actions to perform.
    pub fn update(
        &mut self,
        das: Option<Das<Time>>,
        now: Time,
        keys_down: FrameInput,
    ) -> FrameInput {
        let old = std::mem::replace(&mut self.keys_down, keys_down);
        let new = self.keys_down;

        FrameInput {
            left: self.left.update(das, now, new.left),
            right: self.right.update(das, now, new.right),
            soft_drop: self.soft_drop.update(das, now, new.soft_drop),
            hard_drop: new.hard_drop && !old.hard_drop,
            rot_cw: new.rot_cw && !old.rot_cw,
            rot_ccw: new.rot_ccw && !old.rot_ccw,
            rot_180: new.rot_180 && !old.rot_180,
            hold: new.hold && !old.hold,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum DasState<Time: GameTime> {
    #[default]
    Released,
    Pressed {
        repeat_at: Option<Time>,
    },
}
impl<Time: GameTime> DasState<Time> {
    /// Updates the input state and returns whether to perform the action.
    pub fn update(&mut self, das: Option<Das<Time>>, now: Time, is_down: bool) -> bool {
        if is_down {
            dbg!(&self, now);
            match self {
                DasState::Released => {
                    *self = DasState::Pressed {
                        repeat_at: das.map(|das| now + das.delay),
                    };
                    true
                }

                DasState::Pressed { repeat_at } => {
                    if let Some(das) = das
                        && let Some(repeat_at) = repeat_at
                        && *repeat_at <= now
                    {
                        *repeat_at += das.arr;
                        true
                    } else {
                        false
                    }
                }
            }
        } else {
            *self = Self::Released;
            false
        }
    }
}
