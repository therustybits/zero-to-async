use core::{
    cell::{RefCell, RefMut},
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicU32, Ordering},
    task::{Context, Poll},
};

use critical_section::Mutex;
use fugit::{Duration, Instant};
use heapless::{binary_heap::Min, BinaryHeap};
use microbit::{
    hal::{
        rtc::{RtcCompareReg, RtcInterrupt},
        Rtc,
    },
    pac::{interrupt, NVIC, RTC0},
};

use crate::executor::{wake_task, ExtWaker};

type TickInstant = Instant<u64, 1, 32768>;
type TickDuration = Duration<u64, 1, 32768>;

const MAX_DEADLINES: usize = 8;
static WAKE_DEADLINES: Mutex<RefCell<BinaryHeap<(u64, usize), Min, MAX_DEADLINES>>> =
    Mutex::new(RefCell::new(BinaryHeap::new()));

/// Deadlines can only be scheduled in a COMPARE register if they fall within
/// the current overflow-cycle/epoch, and also are not too close to the current
/// counter value. (see nRF52833 Product Specification section 6.20.7)
fn schedule_wakeup(
    mut rm_deadlines: RefMut<BinaryHeap<(u64, usize), Min, MAX_DEADLINES>>,
    mut rm_rtc: RefMut<Option<Rtc<RTC0>>>,
) {
    let rtc = rm_rtc.as_mut().unwrap();
    while let Some((deadline, task_id)) = rm_deadlines.peek() {
        let ovf_count = (*deadline >> 24) as u32;
        if ovf_count == TICKER.ovf_count.load(Ordering::Relaxed) {
            let counter = (*deadline & 0xFF_FF_FF) as u32;
            if counter > (rtc.get_counter() + 1) {
                rtc.set_compare(RtcCompareReg::Compare0, counter).ok();
                rtc.enable_event(RtcInterrupt::Compare0);
            } else {
                // Wake now if it's too close or already past,
                // then try again with the next available deadline
                wake_task(*task_id);
                rm_deadlines.pop();
                continue;
            }
        }
        break;
    }
    if rm_deadlines.is_empty() {
        rtc.disable_event(RtcInterrupt::Compare0);
    }
}

enum TimerState {
    Init,
    Wait,
}

pub struct Timer {
    end_time: TickInstant,
    state: TimerState,
}

impl Timer {
    pub fn new(duration: TickDuration) -> Self {
        Self {
            end_time: Ticker::now() + duration,
            state: TimerState::Init,
        }
    }

    /// Registration places the deadline & its task_id onto a `BinaryHeap`, and
    /// then will attempt to schedule it (via COMPARE0) if it's earlier than
    /// the current deadline.
    fn register(&self, task_id: usize) {
        let new_deadline = self.end_time.ticks();
        critical_section::with(|cs| {
            let mut rm_deadlines = WAKE_DEADLINES.borrow_ref_mut(cs);
            let is_earliest = if let Some((next_deadline, _)) = rm_deadlines.peek() {
                new_deadline < *next_deadline
            } else {
                true
            };
            if rm_deadlines.push((new_deadline, task_id)).is_err() {
                // Dropping a deadline in this system can be Very Bad:
                //  - In the LED task, the LED will stop updating, but may come
                //    back to life on a button press...
                //  - In a button task, it will never wake again
                // `panic` to raise awareness of the issue during development
                panic!("Deadline dropped for task {}!", task_id);
            }
            // schedule now if its the earliest
            if is_earliest {
                schedule_wakeup(rm_deadlines, TICKER.rtc.borrow_ref_mut(cs));
            }
        });
    }
}

impl Future for Timer {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.state {
            TimerState::Init => {
                self.register(cx.waker().task_id());
                self.state = TimerState::Wait;
                Poll::Pending
            }
            TimerState::Wait => {
                if Ticker::now() >= self.end_time {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            }
        }
    }
}

pub async fn delay(duration: TickDuration) {
    Timer::new(duration).await;
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
    /// interrupt handler function or any `Timer` instance.
    pub fn init(rtc0: RTC0, nvic: &mut NVIC) {
        let mut rtc = Rtc::new(rtc0, 0).unwrap();
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
        rtc.enable_interrupt(RtcInterrupt::Compare0, Some(nvic));
        critical_section::with(|cs| {
            TICKER.rtc.replace(cs, Some(rtc));
        });
    }

    /// Get the current time, which is a combination of:
    ///     - The overflow count (upper 40 bits)
    ///     - The counter value (lowest 24 bits)
    ///
    /// Extra care is needed to ensure the current overflow-count & counter
    /// value are collected during the same overflow-cycle.
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
            TICKER.ovf_count.fetch_add(1, Ordering::Relaxed);
        }
        if rtc.is_event_triggered(RtcInterrupt::Compare0) {
            rtc.reset_event(RtcInterrupt::Compare0);
        }

        // For OVF & COMPARE0 events, schedule the next wakeup. This should also
        // kill enough clock cycles to allow the event flags to clear.
        // (see nRF52833 Product Specification section 6.1.8)
        schedule_wakeup(WAKE_DEADLINES.borrow_ref_mut(cs), rm_rtc);
    });
}
