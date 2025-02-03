#![no_std]
#![no_main]

mod button;
mod led;

use button::ButtonDirection;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{AnyPin, Input, Level, Output, OutputDrive, Pin, Pull};
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
        .spawn(button_task(p.P0_11.degrade(), ButtonDirection::Left))
        .unwrap();
    spawner
        .spawn(button_task(p.P0_12.degrade(), ButtonDirection::Right))
        .unwrap();

    //let _row1 = led_pin(p.P0_21.degrade());
    let col = [
        led_pin(p.P0_13.degrade()),
        led_pin(p.P0_14.degrade()),
        led_pin(p.P0_15.degrade()),
        led_pin(p.P0_16.degrade()),
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

fn led_pin(pin: AnyPin) -> Output<'static> {
    Output::new(pin, Level::High, OutputDrive::Standard)
}

#[embassy_executor::task(pool_size = 2)]
async fn button_task(
    pin: AnyPin,
    direction: ButtonDirection,
) {
    let mut input = Input::new(pin, Pull::None);
    loop {
        input.wait_for_low().await;
        CHANNEL.send(direction).await;
        Timer::after_millis(100).await;
        input.wait_for_high().await;
    }
}
