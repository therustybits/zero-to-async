#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_hal::{
    delay::DelayNs,
    digital::{OutputPin, StatefulOutputPin},
};
use microbit::{hal::Timer, Board};
use panic_halt as _;

#[entry]
fn main() -> ! {
    let mut board = Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);
    let _ = board.display_pins.col1.set_low();
    let mut row1 = board.display_pins.row1;

    loop {
        row1.toggle().ok();
        timer.delay_ms(500);
    }
}
