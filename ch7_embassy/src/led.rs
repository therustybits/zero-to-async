use embassy_nrf::gpio::Output;
use rtt_target::rprintln;

use crate::button::ButtonDirection;

const NUM_COLS: usize = 4;

pub struct LedRow {
    col: [Output<'static>; NUM_COLS],
    active_col: usize,
}

impl LedRow {
    pub fn new(
        col: [Output<'static>; NUM_COLS],
    ) -> Self {
        Self {
            col,
            active_col: 0,
        }
    }

    pub fn shift(&mut self, direction: ButtonDirection) {
        rprintln!("Button press detected..");
        // switch off current/old LED
        self.col[self.active_col].set_high();
        self.active_col = match direction {
            ButtonDirection::Left => match self.active_col {
                0 => NUM_COLS - 1,
                _ => self.active_col - 1,
            }
            ButtonDirection::Right => (self.active_col + 1) % NUM_COLS,
        };
        // switch off new LED: moving to Toggle will then switch it on
        self.col[self.active_col].set_high();
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
        self.col[self.active_col].toggle();
    }
}
