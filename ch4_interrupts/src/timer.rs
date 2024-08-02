use core::{
    cell::RefCell,
    sync::atomic::{AtomicU32, Ordering},
};

use critical_section::Mutex;
use fugit::{Duration, Instant};
use microbit::{
    hal::{rtc::RtcInterrupt, Rtc},
    pac::{interrupt, NVIC, RTC0},
};

type TickInstant = Instant<u64, 1, 32768>;
type TickDuration = Duration<u64, 1, 32768>;

pub struct Timer {
    end_time: TickInstant,
}

impl Timer {
    pub fn new(duration: TickDuration) -> Self {
        Self {
            end_time: Ticker::now() + duration,
        }
    }

    pub fn is_ready(&self) -> bool {
        Ticker::now() >= self.end_time
    }
}

static TICKER: Ticker = Ticker {
    ovf_count: AtomicU32::new(0),
    rtc: Mutex::new(RefCell::new(None)),
};

/// Keeps track of time for the system using RTC0, which ticks away at a rate
/// of 32,768/sec using a low-power oscillator that runs even when the core is
/// powered down.
pub struct Ticker {
    ovf_count: AtomicU32,
    rtc: Mutex<RefCell<Option<Rtc<RTC0>>>>,
}

impl Ticker {
    /// Called on startup to get RTC0 going, then hoists the HAL representation
    /// of RTC0 into the `static TICKER`, where it can be accessed by the
    /// interrupt handler function or any `TickTimer` instance.
    pub fn init(rtc0: RTC0, nvic: &mut NVIC) {
        let mut rtc = Rtc::new(rtc0, 1).unwrap();
        rtc.enable_counter();
        #[cfg(feature = "trigger-overflow")]
        {
            rtc.trigger_overflow();
            // wait for the counter to initialize with its close-to-overflow
            // value before going any further, otherwise one of the tasks could
            // schedule a wakeup that will get skipped over when init happens.
            while rtc.get_counter() == 0 {}
        }
        rtc.enable_event(RtcInterrupt::Overflow);
        rtc.enable_interrupt(RtcInterrupt::Overflow, Some(nvic));
        critical_section::with(|cs| {
            TICKER.rtc.replace(cs, Some(rtc));
        });
    }

    /// Current time is expressed as a TickInstant, which is a combination of:
    ///     - The current RTC0 counter value (lowest 24 bits)
    ///     - RTC0 counter overflow count (`ovf_count`, upper 40 bits)
    ///
    /// Extra care is needed to ensure the current overflow-count & counter
    /// value are collected during the same overflow-cycle.
    /// `Ordering::SeqCst` is used to prevent the compiler or processor from
    /// moving things around.
    pub fn now() -> TickInstant {
        let ticks = {
            loop {
                let ovf_before = TICKER.ovf_count.load(Ordering::SeqCst);
                let counter = critical_section::with(|cs| {
                    TICKER.rtc.borrow_ref(cs).as_ref().unwrap().get_counter()
                });
                let ovf = TICKER.ovf_count.load(Ordering::SeqCst);
                if ovf_before == ovf {
                    break ((ovf as u64) << 24 | counter as u64);
                }
            }
        };
        TickInstant::from_ticks(ticks)
    }
}

#[interrupt]
fn RTC0() {
    critical_section::with(|cs| {
        let mut rm_rtc = TICKER.rtc.borrow_ref_mut(cs);
        let rtc = rm_rtc.as_mut().unwrap();
        if rtc.is_event_triggered(RtcInterrupt::Overflow) {
            rtc.reset_event(RtcInterrupt::Overflow);
            TICKER.ovf_count.fetch_add(1, Ordering::Release);
        }
        // Clearing the event flag can take up to 4 clock cycles:
        // (see nRF52833 Product Specification section 6.1.8)
        // this should do that...
        let _ = rtc.get_counter();
    });
}
