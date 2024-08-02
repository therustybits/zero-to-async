use core::{
    future::poll_fn,
    sync::atomic::{AtomicUsize, Ordering},
    task::Poll,
};

use embedded_hal::digital::{InputPin, PinState};
use microbit::{
    hal::{
        gpio::{Floating, Input, Pin},
        gpiote::Gpiote,
    },
    pac::{interrupt, Interrupt, NVIC},
};

use crate::executor::{wake_task, ExtWaker};

const MAX_CHANNELS: usize = 2;
static NEXT_CHANNEL: AtomicUsize = AtomicUsize::new(0);

pub struct InputChannel {
    pin: Pin<Input<Floating>>,
    channel_id: usize,
}

impl InputChannel {
    pub fn new(pin: Pin<Input<Floating>>, gpiote: &Gpiote) -> Self {
        let channel_id = NEXT_CHANNEL.fetch_add(1, Ordering::Acquire);
        let channel = match channel_id {
            0 => gpiote.channel0(),
            1 => gpiote.channel1(),
            MAX_CHANNELS.. => todo!("Setup more channels!"),
        };
        channel.input_pin(&pin).toggle().enable_interrupt();
        // SAFETY:
        // NVIC not touched in interrupt handlers & PAC knows the value to set.
        unsafe { NVIC::unmask(Interrupt::GPIOTE) };
        Self { pin, channel_id }
    }

    pub async fn wait_for(&mut self, pin_state: PinState) {
        poll_fn(|cx| {
            if pin_state == PinState::from(self.pin.is_high().unwrap()) {
                Poll::Ready(())
            } else {
                WAKE_TASKS[self.channel_id].store(cx.waker().task_id(), Ordering::Relaxed);
                Poll::Pending
            }
        })
        .await
    }
}

const INVALID_TASK_ID: usize = 0xFFFF_FFFF;
const DEFAULT_TASK: AtomicUsize = AtomicUsize::new(INVALID_TASK_ID);
static WAKE_TASKS: [AtomicUsize; MAX_CHANNELS] = [DEFAULT_TASK; MAX_CHANNELS];

#[interrupt]
fn GPIOTE() {
    // SAFETY:
    // Event flags aren't accessed elsewhere
    let gpiote = unsafe { &*microbit::pac::GPIOTE::ptr() };
    for (channel, task) in WAKE_TASKS.iter().enumerate() {
        if gpiote.events_in[channel].read().bits() != 0 {
            gpiote.events_in[channel].write(|w| w);
            // Swap in the INVALID_TASK_ID to prevent the task-ready queue from
            // getting filled up during debounce.
            let task_id = task.swap(INVALID_TASK_ID, Ordering::Acquire);
            if task_id != INVALID_TASK_ID {
                wake_task(task_id);
            }
        }
    }
    // Dummy read to ensure event flags clear
    // (see nRF52833 Product Specification section 6.1.8)
    let _ = gpiote.events_in[0].read().bits();
}
