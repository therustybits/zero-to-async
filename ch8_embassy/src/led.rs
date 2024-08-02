use embassy_nrf::gpio::{AnyPin, Output};
use rtt_target::rprintln;

use crate::button::ButtonDirection;

const NUM_COLS: usize = 5;

pub struct LedRow {
    col: [Output<'static, AnyPin>; NUM_COLS],
    active_col: usize,
}

impl LedRow {
    pub fn new(col: [Output<'static, AnyPin>; NUM_COLS]) -> Self {
        Self { col, active_col: 0 }
    }

    pub fn shift(&mut self, direction: ButtonDirection) {
        rprintln!("Button event received");
        // switch off current/old LED
        self.col[self.active_col].set_high();
        self.active_col = match direction {
            ButtonDirection::Left => match self.active_col {
                0 => 4,
                _ => self.active_col - 1,
            },
            ButtonDirection::Right => (self.active_col + 1) % NUM_COLS,
        };
        // switch off new LED: moving to Toggle will then switch it on
        self.col[self.active_col].set_high();
    }

    pub fn toggle(&mut self) {
        rprintln!("Blinking LED {}", self.active_col);
        self.col[self.active_col].toggle();
    }
}
