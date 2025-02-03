#![no_std]
#![no_main]

mod button;
mod led;

use button::ButtonDirection;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{ Input, Level, Output, OutputDrive,  Pull};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel};
use embassy_time::Timer;
use futures::{select_biased, FutureExt};
use led::LedRow;
use panic_rtt_target as _;
use rtt_target::rtt_init_print;

static CHANNEL: Channel<ThreadModeRawMutex, ButtonDirection, 1> = Channel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    rtt_init_print!();
    let p = embassy_nrf::init(Default::default());
    
    spawner
        .must_spawn(button_task(Input::new(p.P0_11, Pull::Up), ButtonDirection::Left));
    spawner
        .must_spawn(button_task(Input::new(p.P0_12, Pull::Up), ButtonDirection::Right));

    let col = [
        Output::new(p.P0_13, Level::High, OutputDrive::Standard),
        Output::new(p.P0_14, Level::High, OutputDrive::Standard),
        Output::new(p.P0_15, Level::High, OutputDrive::Standard),
        Output::new(p.P0_16, Level::High, OutputDrive::Standard),
        //led_pin(p.P0_30.degrade()),
    ];

    // LED task:
    let mut blinker = LedRow::new(col);
    loop {
        blinker.toggle();
        select_biased! {
            direction = CHANNEL.receive().fuse() => {
                blinker.shift(direction);
            }
            _ = Timer::after_millis(500).fuse() => {}
        }
    }
}

#[embassy_executor::task(pool_size = 2)]
async fn button_task(
    mut input: Input<'static >,
    direction: ButtonDirection,
) {
    loop {
        input.wait_for_low().await;
        CHANNEL.send(direction).await;
        Timer::after_millis(100).await;
        input.wait_for_high().await;
    }
}
