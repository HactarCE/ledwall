use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::animation::step_cyclic_animation;
use crate::widgets::LoadingAnimation;
use crate::{Animation, RED, TintFn, Widget};

/// Loading animation for some data fetched from the web, which becomes stale
/// after a certain period.
///
/// Displays a loading animation while fetching the data, or an error pattern
/// when fetching failed. Nothing is drawn in the case where there is data
/// already fetched.
///
/// If fetching the data fails, it is retried some number of times. If all
/// retries fail, then the data is erased and the loading animation is resumed.
#[derive(Debug)]
pub struct WebLoader<T> {
    config: WebLoaderConfig,

    /// Time to perform the next fetch.
    next_fetch: Option<Instant>,
    /// Number of retries performed since last successful fetch.
    retries: usize,

    request_tx: mpsc::Sender<()>,
    result_rx: mpsc::Receiver<Option<T>>,
    waiting: bool,
    channel_disconnected: bool,

    loaded_value: Option<T>,

    loading_animation: LoadingAnimation,
}

impl<T: 'static + Send> Widget for WebLoader<T> {
    fn step(&mut self, _input: ()) {
        if self.waiting {
            match self.result_rx.try_recv() {
                Ok(Some(new_value)) => {
                    self.waiting = false;
                    self.loaded_value = Some(new_value);
                    self.next_fetch = self.config.refresh_freq.map(|d| Instant::now() + d);
                }
                Ok(None) => {
                    self.waiting = false;
                    self.retries += 1;
                    if 0 < self.config.max_retries && self.config.max_retries <= self.retries {
                        self.invalidate();
                    }
                    self.next_fetch = Some(Instant::now() + self.config.retry_freq);
                }
                Err(mpsc::TryRecvError::Disconnected) => self.channel_disconnected = true,
                Err(mpsc::TryRecvError::Empty) => (), // keep waiting
            }
        } else if self.loaded_value.is_none() || self.next_fetch.is_some_and(|t| t < Instant::now())
        {
            match self.request_tx.send(()) {
                Ok(()) => {
                    self.waiting = true;
                    self.next_fetch = None;
                }
                Err(mpsc::SendError(_)) => self.channel_disconnected = true,
            }
        }

        step_cyclic_animation(&mut self.loading_animation);
    }

    fn draw(&self, fb: &mut crate::FrameBufferRect<'_>) {
        if self.loaded_value.is_some() {
            return;
        }

        if self.channel_disconnected {
            fb.fill(TintFn(
                |[x, y], bg| if (x + y).is_multiple_of(4) { RED } else { bg },
            ));
        } else if self.waiting {
            self.loading_animation.draw(fb, ());
        }
    }
}

impl<T: 'static + Send> WebLoader<T> {
    pub fn new(config: WebLoaderConfig, f: impl 'static + Send + Fn() -> Option<T>) -> Self {
        let (request_tx, request_rx) = mpsc::channel();
        let (result_tx, result_rx) = mpsc::channel();

        std::thread::spawn(move || -> Option<()> {
            loop {
                request_rx.recv().ok()?;
                result_tx.send(f()).ok()?;
            }
        });

        Self {
            config,
            next_fetch: Some(Instant::now()),
            retries: 0,

            request_tx,
            result_rx,
            waiting: false,
            channel_disconnected: false,

            loaded_value: None,

            loading_animation: LoadingAnimation::default(),
        }
    }

    /// Invalidates the existing data and immediately refreshes it.
    pub fn invalidate(&mut self) {
        self.loaded_value = None;
    }

    /// Returns the loaded value, if it has loaded.
    pub fn get(&self) -> Option<&T> {
        self.loaded_value.as_ref()
    }
}

/// Configuration for a [`WebDisplay`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct WebLoaderConfig {
    /// How long to wait before automatically refreshing the display. If this is
    /// `None`, then the display is never automatically refreshed.
    pub refresh_freq: Option<Duration>,
    /// How long to wait between retries.
    pub retry_freq: Duration,
    /// Maximum number of retries before erasing the existing data and returning
    /// to a loading animation. Set this to `0` to never invalidate the data.
    pub max_retries: usize,
}

impl Default for WebLoaderConfig {
    fn default() -> Self {
        Self {
            refresh_freq: Some(Duration::from_secs(5 * 60)),
            retry_freq: Duration::from_secs(5 * 60),
            max_retries: 2,
        }
    }
}
