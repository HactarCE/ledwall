use crate::{
    Activity, AnimationFrame, BLACK, DEFAULT_BRIGHTNESS, DEFAULT_VOLUME, FrameBuffer,
    FrameBufferRect, HEIGHT, Input, Rgb, WHITE, WIDTH, Widget, activities, step_opt_animation,
    widgets,
};

const CONTROLLER_STATUS_BACKGROUND: Rgb = Rgb::from_hex(0x111111);

const BLUE_CONTROLLER_COLOR: Rgb = Rgb::from_hex(0x7777FF);
const GREEN_CONTROLLER_COLOR: Rgb = Rgb::from_hex(0x88DD88);
const DARKEN_DISCONNECTED_CONTROLLER: f32 = 0.5;

const BLUE_CONTROLLER_UUID: [u8; 16] = [5, 0, 0, 0, 200, 45, 0, 0, 32, 144, 0, 0, 0, 1, 0, 0];
const GREEN_CONTROLLER_UUID: [u8; 16] = [3, 0, 0, 0, 200, 45, 0, 0, 32, 144, 0, 0, 0, 1, 0, 0];

const VOLUME_COLOR: Rgb = Rgb::from_hex(0x69F657);
const BRIGHTNESS_COLOR: Rgb = Rgb::from_hex(0xFFE400);

const BACKGROUND_DIM: f32 = 0.75;

#[derive(Debug, Default)]
pub struct ShellFrameOutput {
    pub new_brightness: Option<u8>,
}

/// Graphical shell that provides access to system settings and multiple apps.
pub struct Shell {
    first_frame: std::time::Instant,
    frame_buffer: Box<FrameBuffer>,
    last_input: Input,

    #[cfg(feature = "gilrs")]
    gilrs: gilrs::Gilrs,

    activities: Vec<Box<dyn Activity>>,
    current_activity: usize,

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
            last_input: Input::default(),

            #[cfg(feature = "gilrs")]
            gilrs: gilrs::Gilrs::new().expect("error initializing gamepad"),

            activities: activities::init_activities(),
            current_activity: 0,

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
    pub fn read_gilrs_input(&mut self) -> Input {
        use gilrs::{Axis, Button};

        while self.gilrs.next_event().is_some() {}

        let Some((_id, gamepad)) = self.gilrs.gamepads().next() else {
            return Input::default();
        };

        let x = gamepad.axis_data(Axis::LeftStickX);
        let y = gamepad.axis_data(Axis::LeftStickY);
        let is_button_pressed = |b| gamepad.button_data(b).is_some_and(|d| d.is_pressed());

        Input {
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
        }
    }

    pub fn update(&mut self, input: Input) -> ShellFrameOutput {
        let new_input = input.newly_pressed_compared_to(self.last_input);
        self.last_input = input;

        if new_input.heart {
            self.in_menu ^= true;
            self.menu_animation = Some(match self.menu_animation {
                Some(a) => a.reverse(),
                None => MenuAnimation::new(),
            });
        }
        step_opt_animation(&mut self.menu_animation);

        if self.in_menu {
            let activity_count = self.activities.len();
            if new_input.left {
                self.current_activity =
                    (self.current_activity + activity_count - 1) % activity_count;
            }
            if new_input.right {
                self.current_activity = (self.current_activity + 1) % activity_count;
            }
        }

        let mut fb = FrameBufferRect::new(&mut self.frame_buffer);

        fb.fill(BLACK);

        let activity = &mut self.activities[self.current_activity];
        if !self.in_menu {
            activity.step(input);
        }
        activity.draw(&mut fb);
        if self.in_menu || self.menu_animation.is_some() {
            self.step_and_draw_menu(input)
        } else {
            ShellFrameOutput::default()
        }
    }

    pub fn step_and_draw_menu(&mut self, input: Input) -> ShellFrameOutput {
        let mut output = ShellFrameOutput::default();

        if self.gilrs.gamepads().next().is_none() {
            self.frame_buffer.as_flattened_mut().fill(BLACK);
            return ShellFrameOutput::default();
        }

        let mut blue = false;
        let mut green = false;
        for (_id, gamepad) in self.gilrs.gamepads() {
            blue |= gamepad.uuid() == BLUE_CONTROLLER_UUID;
            green |= gamepad.uuid() == GREEN_CONTROLLER_UUID;
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
        self.activities[self.current_activity]
            .menu_image()
            .draw(&mut upper);

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
            .with_offset([0, 39])
            .with_size([32, 1])
            .fill(Rgb::from_hex(0x666666));

        // Controller status
        {
            let mut fb = lower.with_offset([0, 40]);
            let mut fb = fb.with_size([32, 6]);
            fb.fill(CONTROLLER_STATUS_BACKGROUND);
            include_rgba_image!("controller.rgba").draw_tinted(
                &mut fb.with_offset([5, 1]),
                BLUE_CONTROLLER_COLOR.darken(if blue {
                    0.0
                } else {
                    DARKEN_DISCONNECTED_CONTROLLER
                }),
            );
            include_rgba_image!("controller.rgba").draw_tinted(
                &mut fb.with_offset([19, 1]),
                GREEN_CONTROLLER_COLOR.darken(if green {
                    0.0
                } else {
                    DARKEN_DISCONNECTED_CONTROLLER
                }),
            );
        }

        // Volume slider
        {
            let mut fb = lower.with_offset([0, 46]);
            let mut fb = fb.with_size([32, 9]);
            fb.fill(VOLUME_COLOR.darken(0.85));
            include_rgba_image!("volume.rgba")
                .draw_tinted(&mut fb.with_offset([1, 1]), VOLUME_COLOR);
            include_rgba_image!("l_r.rgba").draw_tinted(&mut fb.with_offset([11, 1]), WHITE);
            self.volume_slider.step([input.l, input.r]);
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
            self.brightness_slider.step([input.lt, input.rt]);
            self.brightness_slider
                .draw(&mut fb.with_offset([11, 6]).with_size([20, 2]));

            let new_brightness = self.brightness_slider.get();
            if new_brightness != old_brightness {
                output.new_brightness = Some(new_brightness)
            }
        }

        output
    }
}

const ARROW_WIGGLE_DURATION: f32 = 2.0;
const ARROW_WIGGLE_DUTY_CYCLE: f32 = 0.25;
const MENU_ANIMATION_DURATION: f32 = 0.125;

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
