use core::cell::Cell;

use embedded_hal::digital::InputPin;
use fugit::ExtU64;
use microbit::hal::gpio::{Floating, Input, Pin};

use crate::timer::{Ticker, Timer};

#[derive(Clone, Copy)]
pub enum ButtonDirection {
    Left,
    Right,
}

enum ButtonState<'a> {
    WaitForPress,
    Debounce(Timer<'a>),
}

pub struct ButtonTask<'a> {
    pin: Pin<Input<Floating>>,
    ticker: &'a Ticker,
    direction: ButtonDirection,
    state: ButtonState<'a>,
    event: &'a Cell<Option<ButtonDirection>>,
}

impl<'a> ButtonTask<'a> {
    pub fn new(
        pin: Pin<Input<Floating>>,
        ticker: &'a Ticker,
        direction: ButtonDirection,
        event: &'a Cell<Option<ButtonDirection>>,
    ) -> Self {
        Self {
            pin,
            ticker,
            direction,
            state: ButtonState::WaitForPress,
            event,
        }
    }

    pub fn poll(&mut self) {
        match self.state {
            ButtonState::WaitForPress => {
                if self.pin.is_low().unwrap() {
                    self.event.set(Some(self.direction));
                    self.state = ButtonState::Debounce(self.ticker.get_timer(100.millis()));
                }
            }
            ButtonState::Debounce(ref timer) => {
                if timer.is_ready() && self.pin.is_high().unwrap() {
                    self.state = ButtonState::WaitForPress;
                }
            }
        }
    }
}
