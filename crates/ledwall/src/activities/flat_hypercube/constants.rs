pub mod colors {
    use crate::color::*;

    pub const STICKERS: [Rgb; 8] = [
        Rgb::from_hex(0xFF0808), // R red
        Rgb::from_hex(0xCC3300), // L orange
        Rgb::from_hex(0xFFFFFF), // U white
        Rgb::from_hex(0xFFFF00), // D yellow
        Rgb::from_hex(0x00FF33), // F green
        Rgb::from_hex(0x0099EE), // B blue
        Rgb::from_hex(0xFF00FF), // O pink
        Rgb::from_hex(0x550088), // I purple
    ];

    pub const INTERNALS: Rgb = Rgb::from_hex(0x111111);
    pub const INTERNALS_SELECTED: Rgb = Rgb::from_hex(0x444444);
    pub const RED_FLASH: Rgb = Rgb::from_hex(0xFF0000);

    pub const DARKEN_UNGRIPPED: f32 = 0.5;

    pub const TIMER_RUNNING: Rgb = Rgb::from_hex(0x999999);
    pub const TIMER_DONE: Rgb = WHITE;
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
