use core::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicUsize, Ordering},
    task::{Context, RawWaker, RawWakerVTable, Waker},
};

use cortex_m::asm;
use heapless::mpmc::Q4;
use rtt_target::rprintln;

/// An alternative to storing the waker: just extract the task information
/// you're looking for via an extension trait that you can implement for `Waker`
/// Not a great general solution if you want to be compatible with other
/// executors, but for this project it's fine.
pub trait ExtWaker {
    fn task_id(&self) -> usize;
}

impl ExtWaker for Waker {
    fn task_id(&self) -> usize {
        // When "waker-getters" is stabilized, do this instead:
        // self.as_raw().data() as usize
        for task_id in 0..NUM_TASKS.load(Ordering::Relaxed) {
            if get_waker(task_id).will_wake(self) {
                return task_id;
            }
        }
        panic!("Unknown waker/executor!");
    }
}

fn get_waker(task_id: usize) -> Waker {
    // SAFETY:
    // Data argument interpreted as an integer, not dereferenced
    unsafe {
        Waker::from_raw(RawWaker::new(task_id as *const (), &VTABLE))
    }
}

static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

unsafe fn clone(p: *const ()) -> RawWaker {
    RawWaker::new(p, &VTABLE)
}

unsafe fn drop(_p: *const ()) {}

unsafe fn wake(p: *const ()) {
    wake_task(p as usize);
}

unsafe fn wake_by_ref(p: *const ()) {
    wake_task(p as usize);
}

pub fn wake_task(task_id: usize) {
    rprintln!("Waking task {}", task_id);
    if TASK_ID_READY.enqueue(task_id).is_err() {
        // Being unable to wake a task will likely cause it to become
        // permanently unresponsive.
        panic!("Task queue full: can't add task {}", task_id);
    }
}

static TASK_ID_READY: Q4<usize> = Q4::new();
static NUM_TASKS: AtomicUsize = AtomicUsize::new(0);

pub fn run_tasks(tasks: &mut [Pin<&mut dyn Future<Output = ()>>]) -> ! {
    NUM_TASKS.store(tasks.len(), Ordering::Relaxed);

    // everybody gets one run to start...
    for task_id in 0..tasks.len() {
        TASK_ID_READY.enqueue(task_id).ok();
    }

    loop {
        while let Some(task_id) = TASK_ID_READY.dequeue() {
            if task_id >= tasks.len() {
                rprintln!("Bad task id {}!", task_id);
                continue;
            }
            rprintln!("Running task {}", task_id);
            let _ = tasks[task_id]
                .as_mut()
                .poll(&mut Context::from_waker(&get_waker(task_id)));
        }
        rprintln!("No tasks ready, going to sleep...");
        asm::wfi();
    }
}
