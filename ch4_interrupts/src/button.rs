use core::cell::Cell;

use embedded_hal::digital::InputPin;
use fugit::ExtU64;
use microbit::hal::gpio::{Floating, Input, Pin};

use crate::time::Timer;

#[derive(Clone, Copy)]
pub enum ButtonDirection {
    Left,
    Right,
}

enum ButtonState {
    WaitForPress,
    Debounce(Timer),
}

pub struct ButtonTask<'a> {
    pin: Pin<Input<Floating>>,
    direction: ButtonDirection,
    state: ButtonState,
    event: &'a Cell<Option<ButtonDirection>>,
}

impl<'a> ButtonTask<'a> {
    pub fn new(
        pin: Pin<Input<Floating>>,
        direction: ButtonDirection,
        event: &'a Cell<Option<ButtonDirection>>,
    ) -> Self {
        Self {
            pin,
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
                    self.state = ButtonState::Debounce(Timer::new(100.millis()));
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
