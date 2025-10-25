#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Input {
    // D pad
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,

    // Thumb buttons
    pub a: bool,
    pub b: bool,
    pub x: bool,
    pub y: bool,

    // Shoulder buttons
    pub l: bool,
    pub r: bool,
    pub lt: bool,
    pub rt: bool,

    // Middle buttons
    pub plus: bool,
    pub minus: bool,
    pub star: bool,
    pub heart: bool,
}
