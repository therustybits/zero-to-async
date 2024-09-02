use embedded_hal::digital::{OutputPin, StatefulOutputPin};
use microbit::{
    gpio::NUM_COLS,
    hal::gpio::{Output, Pin, PushPull},
};
use rtt_target::rprintln;

use crate::button::ButtonDirection;

pub struct LedRow {
    col: [Pin<Output<PushPull>>; NUM_COLS],
    active_col: usize,
}

impl LedRow {
    pub fn new(
        col: [Pin<Output<PushPull>>; NUM_COLS],
    ) -> Self {
        Self {
            col,
            active_col: 0,
        }
    }

    pub fn shift(&mut self, direction: ButtonDirection) {
        rprintln!("Button press detected..");
        // switch off current/old LED
        self.col[self.active_col].set_high().ok();
        self.active_col = match direction {
            ButtonDirection::Left => match self.active_col {
                0 => 4,
                _ => self.active_col - 1,
            }
            ButtonDirection::Right => (self.active_col + 1) % NUM_COLS,
        };
        // switch off new LED: moving to Toggle will then switch it on
        self.col[self.active_col].set_high().ok();
    }

    pub fn toggle(&mut self) {
        rprintln!("Blinking LED {}", self.active_col);
        #[cfg(feature = "trigger-overflow")]
        {
            use crate::time::Ticker;
            let time = Ticker::now();
            rprintln!(
                "Time: 0x{:x} ticks, {} ms",
                time.ticks(),
                time.duration_since_epoch().to_millis(),
            );
        }
        self.col[self.active_col].toggle().ok();
    }
}
