#![no_std]
#![no_main]

mod button;
mod channel;
mod led;
mod time;

use button::{ButtonDirection, ButtonTask};
use channel::Channel;
use cortex_m_rt::entry;
use embedded_hal::digital::OutputPin;
use led::LedTask;
use microbit::Board;
use panic_rtt_target as _;
use rtt_target::rtt_init_print;
use time::Ticker;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let mut board = Board::take().unwrap();
    Ticker::init(board.RTC0, &mut board.NVIC);
    let (col, mut row) = board.display_pins.degrade();
    row[0].set_high().ok();
    let button_l = board.buttons.button_a.degrade();
    let button_r = board.buttons.button_b.degrade();

    let channel: Channel<ButtonDirection> = Channel::new();
    let mut led_task = LedTask::new(col, channel.get_receiver());
    let mut button_l_task = ButtonTask::new(button_l, ButtonDirection::Left, channel.get_sender());
    let mut button_r_task = ButtonTask::new(button_r, ButtonDirection::Right, channel.get_sender());

    loop {
        led_task.poll();
        button_l_task.poll();
        button_r_task.poll();
    }
}
