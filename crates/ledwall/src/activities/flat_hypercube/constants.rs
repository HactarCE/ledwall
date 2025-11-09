pub mod colors {
    use crate::color::*;

    pub const STICKERS: [Rgb; 8] = [
        Rgb::from_hex(0xFF0808), // R red
        Rgb::from_hex(0xCC5500), // L orange
        Rgb::from_hex(0xFFFFFF), // U white
        Rgb::from_hex(0xFFFF00), // D yellow
        Rgb::from_hex(0x33CC33), // F green
        Rgb::from_hex(0x0099EE), // B blue
        Rgb::from_hex(0xFF2266), // O pink
        Rgb::from_hex(0x550088), // I purple
    ];

    pub const INTERNALS: Rgb = Rgb::from_hex(0x111111);
    pub const INTERNALS_SELECTED: Rgb = Rgb::from_hex(0x444444);
    pub const RED_FLASH: Rgb = Rgb::from_hex(0xFF0000);

    pub const DARKEN_UNGRIPPED: f32 = 0.5;
    pub const DARKEN_HIDDEN: f32 = 0.5;

    pub const TIMER_RUNNING: Rgb = Rgb::from_hex(0x999999);
    pub const TIMER_DONE: Rgb = WHITE;

    pub const FILTER_MUST_NOT_HAVE: Rgb = Rgb::from_hex(0x000000);
    pub const FILTER_MAY_HAVE: Rgb = Rgb::from_hex(0x222222);
    pub const FILTER_MUST_HAVE: Rgb = Rgb::from_hex(0x999999);
}

pub mod animations {
    pub mod blink {
        pub const DURATION: f32 = 1.0; // seconds
    }

    pub mod red_flash {
        pub const DURATION: f32 = 0.25; // seconds
    }

    pub mod turn {
        pub const DURATION: f32 = 0.5; // seconds
    }
}
