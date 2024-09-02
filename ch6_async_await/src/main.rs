#![no_std]
#![no_main]

mod button;
mod channel;
mod executor;
mod gpiote;
mod led;
mod time;

use core::pin::pin;

use button::ButtonDirection;
use channel::{Channel, Receiver, Sender};
use cortex_m_rt::entry;
use embedded_hal::digital::{OutputPin, PinState};
use fugit::ExtU64;
use futures::{select_biased, FutureExt};
use gpiote::InputChannel;
use led::LedRow;
use microbit::{
    gpio::NUM_COLS,
    hal::{
        gpio::{Floating, Input, Output, Pin, PushPull},
        gpiote::Gpiote,
    },
    Board,
};
use panic_rtt_target as _;
use rtt_target::rtt_init_print;
use time::Ticker;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let mut board = Board::take().unwrap();
    Ticker::init(board.RTC0, &mut board.NVIC);
    let gpiote = Gpiote::new(board.GPIOTE);
    let (col, mut row) = board.display_pins.degrade();
    row[0].set_high().ok();
    let button_l = board.buttons.button_a.degrade();
    let button_r = board.buttons.button_b.degrade();

    let channel: Channel<ButtonDirection> = Channel::new();
    let led_task = pin!(led_task(col, channel.get_receiver()));
    let button_l_task = pin!(button_task(
        button_l,
        ButtonDirection::Left,
        channel.get_sender(),
        &gpiote
    ));
    let button_r_task = pin!(button_task(
        button_r,
        ButtonDirection::Right,
        channel.get_sender(),
        &gpiote
    ));

    executor::run_tasks(&mut [led_task, button_l_task, button_r_task]);
}

async fn led_task(
    col: [Pin<Output<PushPull>>; NUM_COLS],
    mut receiver: Receiver<'_, ButtonDirection>,
) {
    let mut blinker = LedRow::new(col);
    loop {
        blinker.toggle();
        select_biased! {
            direction = receiver.receive().fuse() => {
                blinker.shift(direction);
            }
            _ = time::delay(500.millis()).fuse() => {}
        }
    }
}

async fn button_task(
    pin: Pin<Input<Floating>>,
    direction: ButtonDirection,
    sender: Sender<'_, ButtonDirection>,
    gpiote: &Gpiote,
) {
    let mut input = InputChannel::new(pin, gpiote);
    loop {
        input.wait_for(PinState::Low).await;
        sender.send(direction);
        time::delay(100.millis()).await;
        input.wait_for(PinState::High).await;
    }
}
