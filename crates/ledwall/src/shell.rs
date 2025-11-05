use crate::{
    Activity, AnimationFrame, BLACK, Buttons, ControllerInput, DEFAULT_BRIGHTNESS, DEFAULT_VOLUME,
    FrameBuffer, FrameBufferRect, FullInput, HEIGHT, Rgb, WHITE, WIDTH, Widget, activities,
    map_range, step_opt_animation, widgets,
};

const CONTROLLER_STATUS_BACKGROUND: Rgb = BLACK;

const BLUE_CONTROLLER_COLOR: Rgb = Rgb::from_hex(0x7777FF);
const GREEN_CONTROLLER_COLOR: Rgb = Rgb::from_hex(0x88DD88);
const DARKEN_DISCONNECTED_CONTROLLER: f32 = 0.75;

#[cfg(feature = "gilrs")]
const BLUE_CONTROLLER_UUID: [u8; 16] = [5, 0, 0, 0, 200, 45, 0, 0, 32, 144, 0, 0, 0, 1, 0, 0];
#[cfg(feature = "gilrs")]
const GREEN_CONTROLLER_UUID: [u8; 16] = [3, 0, 0, 0, 200, 45, 0, 0, 32, 144, 0, 0, 0, 1, 0, 0];

const VOLUME_COLOR: Rgb = Rgb::from_hex(0x69F657);
const BRIGHTNESS_COLOR: Rgb = Rgb::from_hex(0xFFE400);

const BACKGROUND_DIM: f32 = 0.875;

#[derive(Debug, Default)]
pub struct ShellFrameOutput {
    pub new_brightness: Option<u8>,
}

/// Graphical shell that provides access to system settings and multiple apps.
pub struct Shell {
    first_frame: std::time::Instant,
    frame_buffer: Box<FrameBuffer>,
    last_input_blue: Option<Buttons>,
    last_input_green: Option<Buttons>,

    #[cfg(feature = "gilrs")]
    gilrs: gilrs::Gilrs,

    activities: Vec<Box<dyn Activity>>,
    current_activity: usize,
    activity_reset_animation: Option<ActivityResetAnimation>,

    /// Volume on a scale from 0 to 20
    volume_slider: widgets::Slider,
    /// Brightness on a scale from 0 to 20
    brightness_slider: widgets::Slider,

    in_menu: bool,
    menu_animation: Option<MenuAnimation>,
}
impl Default for Shell {
    fn default() -> Self {
        Self {
            first_frame: std::time::Instant::now(),
            frame_buffer: Box::new([[BLACK; WIDTH]; HEIGHT]),
            last_input_blue: None,
            last_input_green: None,

            #[cfg(feature = "gilrs")]
            gilrs: gilrs::Gilrs::new().expect("error initializing gamepad"),

            activities: activities::init_activities(),
            current_activity: 0,
            activity_reset_animation: None,

            volume_slider: widgets::Slider::new(DEFAULT_VOLUME, 0..=20, VOLUME_COLOR),
            brightness_slider: widgets::Slider::new(DEFAULT_BRIGHTNESS, 1..=20, BRIGHTNESS_COLOR),

            in_menu: true,
            menu_animation: None,
        }
    }
}

impl Shell {
    pub fn frame_buffer(&self) -> &FrameBuffer {
        &self.frame_buffer
    }

    #[cfg(feature = "gilrs")]
    pub fn read_gilrs_input(&mut self) -> (Option<Buttons>, Option<Buttons>) {
        use gilrs::{Axis, Button};

        // Process gilrs events
        while let Some(ev) = self.gilrs.next_event() {
            if let gilrs::EventType::Connected = ev.event
                && let Some(gamepad) = self.gilrs.connected_gamepad(ev.id)
            {
                let uuid = gamepad.uuid();
                println!("Connected controller with UUID {uuid:?}");
            }
        }

        let mut blue = None;
        let mut green = None;

        for (_id, gamepad) in self.gilrs.gamepads() {
            let x = gamepad.axis_data(Axis::LeftStickX);
            let y = gamepad.axis_data(Axis::LeftStickY);
            let is_button_pressed = |b| gamepad.button_data(b).is_some_and(|d| d.is_pressed());
            let current_button_states = Buttons {
                up: y.is_some_and(|y| y.value() > 0.5),
                down: y.is_some_and(|y| y.value() < -0.5),
                left: x.is_some_and(|x| x.value() < -0.5),
                right: x.is_some_and(|x| x.value() > 0.5),
                a: is_button_pressed(Button::East),
                b: is_button_pressed(Button::South),
                x: is_button_pressed(Button::North),
                y: is_button_pressed(Button::West),
                l: is_button_pressed(Button::LeftTrigger),
                r: is_button_pressed(Button::RightTrigger),
                lt: is_button_pressed(Button::LeftTrigger2),
                rt: is_button_pressed(Button::RightTrigger2),
                plus: is_button_pressed(Button::Start),
                minus: is_button_pressed(Button::Select),
                star: false, // can't access
                heart: is_button_pressed(Button::Mode),
            };

            *match gamepad.uuid() {
                BLUE_CONTROLLER_UUID => &mut blue,
                GREEN_CONTROLLER_UUID => &mut green,
                _ => &mut green, // fallback for unrecognized controller
            } = Some(current_button_states);
        }

        (blue, green)
    }

    pub fn update(&mut self, blue: Option<Buttons>, green: Option<Buttons>) -> ShellFrameOutput {
        let prev_blue = std::mem::replace(&mut self.last_input_blue, blue);
        let prev_green = std::mem::replace(&mut self.last_input_green, green);

        let full_input = FullInput {
            blue: blue.map(|current| {
                let previous = prev_blue.unwrap_or_default();
                ControllerInput { current, previous }
            }),
            green: green.map(|current| {
                let previous = prev_green.unwrap_or_default();
                ControllerInput { current, previous }
            }),
        };

        if blue.is_none() && green.is_none() {
            self.frame_buffer.as_flattened_mut().fill(BLACK);
            self.current_activity = 0;
            self.in_menu = false;
            self.menu_animation = None;
            return ShellFrameOutput::default();
        }

        let pressed_keys = full_input.any().pressed();

        if pressed_keys.heart || (self.in_menu && pressed_keys.a) {
            self.toggle_menu();
        }
        if self.in_menu && pressed_keys.x {
            self.activities[self.current_activity].reset();
            self.activity_reset_animation = Some(ActivityResetAnimation::new())
        }
        step_opt_animation(&mut self.menu_animation);
        step_opt_animation(&mut self.activity_reset_animation);

        if self.in_menu {
            let activity_count = self.activities.len();
            if pressed_keys.left {
                self.current_activity =
                    (self.current_activity + activity_count - 1) % activity_count;
                self.activity_reset_animation = None;
            }
            if pressed_keys.right {
                self.current_activity = (self.current_activity + 1) % activity_count;
                self.activity_reset_animation = None;
            }
        }

        let mut fb = FrameBufferRect::new(&mut self.frame_buffer);

        fb.fill(BLACK);

        let activity = &mut self.activities[self.current_activity];
        if !self.in_menu && self.menu_animation.is_none() {
            activity.step(full_input);
        }
        activity.draw(&mut fb);
        if self.in_menu || self.menu_animation.is_some() {
            self.step_and_draw_menu(full_input.any())
        } else {
            ShellFrameOutput::default()
        }
    }

    pub fn toggle_menu(&mut self) {
        self.in_menu ^= true;
        self.menu_animation = Some(match self.menu_animation {
            Some(a) => a.reverse(),
            None => MenuAnimation::new(),
        });
    }

    pub fn step_and_draw_menu(&mut self, input: ControllerInput) -> ShellFrameOutput {
        let mut output = ShellFrameOutput::default();

        #[allow(unused_mut)]
        let mut blue = false;
        #[allow(unused_mut)]
        let mut green = false;
        #[cfg(feature = "gilrs")]
        {
            for (_id, gamepad) in self.gilrs.gamepads() {
                blue |= gamepad.uuid() == BLUE_CONTROLLER_UUID;
                green |= gamepad.uuid() == GREEN_CONTROLLER_UUID;
            }
        }

        let mut fb = FrameBufferRect::new(&mut self.frame_buffer);
        let mut t = match self.menu_animation {
            Some(a) => a.t(),
            None => 1.0,
        };
        if self.in_menu {
            t = 1.0 - t;
        }
        t *= t;

        // Dim background
        fb.fill_with_fn(|_, color| color.darken(BACKGROUND_DIM * (1.0 - t)));

        let mut upper = fb.with_offset([0, (fb.height() as f32 / 2.0 * -t) as isize]);

        // Activity menu image
        let darken = match self.activity_reset_animation {
            Some(anim) => map_range(anim.t(), 0.5..1.0, 0.5..0.0),
            None => 0.0,
        };
        self.activities[self.current_activity]
            .menu_image()
            .draw_with_custom_blend(&mut upper, |c1, c2, alpha| {
                c1.mix(
                    c2.darken(darken),
                    (alpha as f32 / 255.0) * map_range(t, 0.0..0.125, 1.0..0.0),
                )
            });

        // Activity selection arrows
        const NANOS_PER_SEC: f32 = 1_000_000_000 as f32;
        let t2 = (self.first_frame.elapsed().as_nanos()
            % (ARROW_WIGGLE_DURATION * NANOS_PER_SEC) as u128) as f32
            / NANOS_PER_SEC;
        let wiggle = t2 < ARROW_WIGGLE_DUTY_CYCLE;
        include_rgba_image!("arrow_left.rgba")
            .draw(&mut upper.with_offset([1 - wiggle as isize, 3]));
        include_rgba_image!("arrow_right.rgba")
            .draw(&mut upper.with_offset([26 + wiggle as isize, 3]));

        let mut lower = fb.with_offset([0, (fb.height() as f32 / 2.0 * t) as isize]);

        // Border line
        lower
            .with_offset([0, 36])
            .with_size([32, 1])
            .fill(Rgb::from_hex(0x666666));

        // Controller status
        {
            let mut fb = lower.with_offset([0, 37]);
            let mut fb = fb.with_size([32, 9]);
            fb.fill(CONTROLLER_STATUS_BACKGROUND);
            let blue_darken = if blue {
                0.0
            } else {
                DARKEN_DISCONNECTED_CONTROLLER
            };
            include_rgba_image!("controller.rgba").draw_tinted(
                &mut fb.with_offset([1, 1]),
                BLUE_CONTROLLER_COLOR.darken(blue_darken),
            );
            include_rgba_image!("controller_buttons.rgba")
                .draw_tinted(&mut fb.with_offset([1, 1]), WHITE.darken(blue_darken));

            let green_darken = if green {
                0.0
            } else {
                DARKEN_DISCONNECTED_CONTROLLER
            };
            include_rgba_image!("controller.rgba").draw_tinted(
                &mut fb.with_offset([17, 1]),
                GREEN_CONTROLLER_COLOR.darken(green_darken),
            );
            include_rgba_image!("controller_buttons.rgba")
                .draw_tinted(&mut fb.with_offset([17, 1]), WHITE.darken(green_darken));
        }

        // Volume slider
        {
            let mut fb = lower.with_offset([0, 46]);
            let mut fb = fb.with_size([32, 9]);
            fb.fill(VOLUME_COLOR.darken(0.85));
            include_rgba_image!("volume.rgba")
                .draw_tinted(&mut fb.with_offset([1, 1]), VOLUME_COLOR);
            include_rgba_image!("l_r.rgba").draw_tinted(&mut fb.with_offset([11, 1]), WHITE);
            self.volume_slider
                .step([input.pressed().l, input.pressed().r]);
            self.volume_slider
                .draw(&mut fb.with_offset([11, 6]).with_size([20, 2]));
        }

        // Brightness slider
        {
            let old_brightness = self.brightness_slider.get();

            let mut fb = lower.with_offset([0, 55]);
            fb.fill(BRIGHTNESS_COLOR.darken(0.85));
            include_rgba_image!("brightness.rgba")
                .draw_tinted(&mut fb.with_offset([1, 1]), BRIGHTNESS_COLOR);
            include_rgba_image!("l2_r2.rgba").draw_tinted(&mut fb.with_offset([11, 1]), WHITE);
            self.brightness_slider
                .step([input.pressed().lt, input.pressed().rt]);
            self.brightness_slider
                .draw(&mut fb.with_offset([11, 6]).with_size([20, 2]));

            let new_brightness = self.brightness_slider.get();
            if new_brightness != old_brightness {
                output.new_brightness = Some(new_brightness);
            }
        }

        output
    }
}

const ARROW_WIGGLE_DURATION: f32 = 2.0;
const ARROW_WIGGLE_DUTY_CYCLE: f32 = 0.25;
const MENU_ANIMATION_DURATION: f32 = 0.25;

#[derive(Debug, Default, Copy, Clone)]
struct MenuAnimation {
    frame: u32,
}
impl_animation_frame!(MenuAnimation, MENU_ANIMATION_DURATION);
impl MenuAnimation {
    pub fn new() -> Self {
        Self { frame: 0 }
    }

    pub fn reverse(self) -> Self {
        let frame_count = (MENU_ANIMATION_DURATION * crate::FPS as f32) as u32;
        Self {
            frame: frame_count.saturating_sub(self.frame),
        }
    }
}

const ACTIVITY_RESET_ANIMATION_DURATION: f32 = 1.0;

#[derive(Debug, Default, Copy, Clone)]
struct ActivityResetAnimation {
    frame: u32,
}
impl_animation_frame!(ActivityResetAnimation, ACTIVITY_RESET_ANIMATION_DURATION);
impl ActivityResetAnimation {
    pub fn new() -> Self {
        Self { frame: 0 }
    }
}
