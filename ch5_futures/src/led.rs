use embedded_hal::digital::{OutputPin, StatefulOutputPin};
use fugit::ExtU64;
use microbit::{
    gpio::NUM_COLS,
    hal::gpio::{Output, Pin, PushPull},
};
use rtt_target::rprintln;

use crate::{
    button::ButtonDirection,
    channel::Receiver,
    future::{OurFuture, Poll},
    time::Timer,
};

enum LedState {
    Toggle,
    Wait(Timer),
}

pub struct LedTask<'a> {
    col: [Pin<Output<PushPull>>; NUM_COLS],
    active_col: usize,
    state: LedState,
    receiver: Receiver<'a, ButtonDirection>,
}

impl<'a> LedTask<'a> {
    pub fn new(
        col: [Pin<Output<PushPull>>; NUM_COLS],
        receiver: Receiver<'a, ButtonDirection>,
    ) -> Self {
        Self {
            col,
            active_col: 0,
            state: LedState::Toggle,
            receiver,
        }
    }

    fn shift(&mut self, direction: ButtonDirection) {
        rprintln!("Button press detected..");
        // switch off current/old LED
        self.col[self.active_col].set_high().ok();
        self.active_col = match direction {
            ButtonDirection::Left => match self.active_col {
                0 => 4,
                _ => self.active_col - 1,
            }
            ButtonDirection::Right => (self.active_col + 1) % NUM_COLS,
        };
        // switch off new LED: moving to Toggle will then switch it on
        self.col[self.active_col].set_high().ok();
    }

    fn toggle(&mut self) {
        rprintln!("Blinking LED {}", self.active_col);
        #[cfg(feature = "trigger-overflow")]
        {
            use crate::time::Ticker;
            let time = Ticker::now();
            rprintln!(
                "Time: 0x{:x} ticks, {} ms",
                time.ticks(),
                time.duration_since_epoch().to_millis(),
            );
        }
        self.col[self.active_col].toggle().ok();
    }
}

impl OurFuture for LedTask<'_> {
    type Output = ();
    fn poll(&mut self, task_id: usize) -> Poll<Self::Output> {
        loop {
            match self.state {
                LedState::Toggle => {
                    self.toggle();
                    self.state = LedState::Wait(Timer::new(500.millis()));
                }
                LedState::Wait(ref mut timer) => {
                    if let Poll::Ready(_) = timer.poll(task_id) {
                        self.state = LedState::Toggle;
                        continue;
                    }
                    if let Poll::Ready(direction) = self.receiver.poll(task_id) {
                        self.shift(direction);
                        self.state = LedState::Toggle;
                        continue;
                    }
                    break;
                }
            }
        }
        Poll::Pending
    }
}
