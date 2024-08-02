use fugit::{Duration, Instant};
use microbit::{hal::Rtc, pac::RTC0};

type TickInstant = Instant<u64, 1, 32768>;
type TickDuration = Duration<u64, 1, 32768>;

pub struct Timer<'a> {
    ticker: &'a Ticker,
    end_time: TickInstant,
}

impl<'a> Timer<'a> {
    fn new(duration: TickDuration, ticker: &'a Ticker) -> Self {
        Self {
            ticker,
            end_time: ticker.now() + duration,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.ticker.now() >= self.end_time
    }
}

/// Keeps track of time for the system using RTC0, which ticks away at a rate
/// of 32,768/sec using a low-power oscillator that runs even when the core is
/// powered down.
///
/// RTC0's counter is only 24-bits wide, which means there will be an overflow
/// every ~8min, which we do not account for: this will be fixed in chapter 4.
pub struct Ticker {
    rtc: Rtc<RTC0>,
}

impl Ticker {
    /// Create on startup to get RTC0 going.
    pub fn new(rtc0: RTC0) -> Self {
        let rtc = Rtc::new(rtc0, 1).unwrap();
        rtc.enable_counter();
        Self { rtc }
    }

    pub fn now(&self) -> TickInstant {
        TickInstant::from_ticks(self.rtc.get_counter() as u64)
    }

    pub fn get_timer(&self, duration: TickDuration) -> Timer {
        Timer::new(duration, &self)
    }
}
