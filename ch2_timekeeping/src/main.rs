#![no_std]
#![no_main]

mod time;

use cortex_m_rt::entry;
use embedded_hal::{
    delay::DelayNs,
    digital::{InputPin, OutputPin, StatefulOutputPin},
};
use microbit::{hal::Timer, Board};
use panic_rtt_target as _;
use rtt_target::rtt_init_print;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);
    let (mut col, mut row) = board.display_pins.degrade();
    row[0].set_high().ok();
    let mut button_l = board.buttons.button_a.degrade();
    let mut button_r = board.buttons.button_b.degrade();

    let active_col: usize = 0;
    loop {
        col[active_col].toggle().ok();
        // blocking here:
        timer.delay_ms(500);
        // will prevent timely detection & response to these:
        if button_l.is_low().unwrap() {
            //..
        }
        if button_r.is_low().unwrap() {
            //..
        }
    }
}
