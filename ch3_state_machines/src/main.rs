#![no_std]
#![no_main]

mod button;
mod led;
mod timer;

use core::cell::Cell;

use button::{ButtonDirection, ButtonTask};
use cortex_m_rt::entry;
use embedded_hal::digital::OutputPin;
use led::LedTask;
use microbit::Board;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use timer::Ticker;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = Board::take().unwrap();
    let ticker = Ticker::new(board.RTC0);
    let (col, mut row) = board.display_pins.degrade();
    row[0].set_high().ok();
    let button_l = board.buttons.button_a.degrade();
    let button_r = board.buttons.button_b.degrade();

    let button_event: Cell<Option<ButtonDirection>> = Cell::new(None);
    let mut led_task = LedTask::new(col, &ticker, &button_event);
    let mut button_l_task =
        ButtonTask::new(button_l, &ticker, ButtonDirection::Left, &button_event);
    let mut button_r_task =
        ButtonTask::new(button_r, &ticker, ButtonDirection::Right, &button_event);

    rprintln!("Running tasks...");
    loop {
        led_task.poll();
        button_l_task.poll();
        button_r_task.poll();
    }
}
