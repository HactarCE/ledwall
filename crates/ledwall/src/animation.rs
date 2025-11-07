use crate::FrameBufferRect;

macro_rules! impl_animation_frame {
    ($ty:ty, $duration:expr $(,)?) => {
        impl $crate::animation::AnimationFrame for $ty {
            const DURATION: f32 = $duration;

            fn frame_number(&self) -> u32 {
                self.frame
            }
            fn frame_number_mut(&mut self) -> &mut u32 {
                &mut self.frame
            }
        }
    };
}

pub trait AnimationFrame {
    /// Animation duration in seconds.
    const DURATION: f32;

    /// Returns the frame number of the animation.
    fn frame_number(&self) -> u32;
    /// Returns a mutable reference to the frame number of the animation.
    fn frame_number_mut(&mut self) -> &mut u32;

    /// Returns the progress into the animation.
    fn t(&self) -> f32 {
        self.frame_number() as f32 / (Self::DURATION * crate::FPS as f32)
    }
}

pub trait Animation<D>: AnimationFrame {
    fn draw(&self, fb: &mut FrameBufferRect<'_>, data: D);
}

pub fn step_opt_animation<A: AnimationFrame>(opt_anim: &mut Option<A>) -> Option<A> {
    opt_anim.take_if(|anim| {
        *anim.frame_number_mut() += 1;
        anim.t() > 1.0
    })
}

pub fn draw_opt_animation<D>(
    opt_anim: &Option<impl Animation<D>>,
    fb: &mut FrameBufferRect<'_>,
    data: D,
) {
    if let Some(anim) = opt_anim {
        anim.draw(fb, data);
    }
}
