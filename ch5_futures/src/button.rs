use embedded_hal::digital::PinState;
use fugit::ExtU64;
use microbit::hal::{
    gpio::{Floating, Input, Pin},
    gpiote::Gpiote,
};

use crate::{
    channel::Sender,
    future::{OurFuture, Poll},
    gpiote::InputChannel,
    time::Timer,
};

#[derive(Clone, Copy)]
pub enum ButtonDirection {
    Left,
    Right,
}

enum ButtonState {
    WaitForPress,
    Debounce(Timer),
    WaitForRelease,
}

pub struct ButtonTask<'a> {
    input: InputChannel,
    direction: ButtonDirection,
    state: ButtonState,
    sender: Sender<'a, ButtonDirection>,
}

impl<'a> ButtonTask<'a> {
    pub fn new(
        pin: Pin<Input<Floating>>,
        direction: ButtonDirection,
        sender: Sender<'a, ButtonDirection>,
        gpiote: &Gpiote,
    ) -> Self {
        Self {
            input: InputChannel::new(pin, gpiote),
            direction,
            state: ButtonState::WaitForPress,
            sender,
        }
    }
}

impl OurFuture for ButtonTask<'_> {
    type Output = ();
    fn poll(&mut self, task_id: usize) -> Poll<Self::Output> {
        loop {
            match self.state {
                ButtonState::WaitForPress => {
                    self.input.set_ready_state(PinState::Low);
                    match self.input.poll(task_id) {
                        Poll::Ready(_) => {
                            self.sender.send(self.direction);
                            self.state = ButtonState::Debounce(Timer::new(100.millis()))
                        }
                        Poll::Pending => break,
                    }
                }
                ButtonState::Debounce(ref mut timer) => match timer.poll(task_id) {
                    Poll::Ready(_) => self.state = ButtonState::WaitForRelease,
                    Poll::Pending => break,
                },
                ButtonState::WaitForRelease => {
                    self.input.set_ready_state(PinState::High);
                    match self.input.poll(task_id) {
                        Poll::Ready(_) => self.state = ButtonState::WaitForPress,
                        Poll::Pending => break,
                    }
                }
            }
        }
        Poll::Pending
    }
}
