use cortex_m::asm;
use heapless::mpmc::Q4;
use rtt_target::rprintln;

use crate::future::OurFuture;

pub fn wake_task(task_id: usize) {
    rprintln!("Waking task {}", task_id);
    if TASK_ID_READY.enqueue(task_id).is_err() {
        // Being unable to wake a task will likely cause it to become
        // permanently unresponsive...
        panic!("Task queue full: can't add task {}", task_id);
    }
}

static TASK_ID_READY: Q4<usize> = Q4::new();

pub fn run_tasks(tasks: &mut [&mut dyn OurFuture<Output = ()>]) -> ! {
    // everybody gets one run to start...
    for task_id in 0..tasks.len() {
        wake_task(task_id);
    }

    loop {
        while let Some(task_id) = TASK_ID_READY.dequeue() {
            if task_id >= tasks.len() {
                rprintln!("Bad task id {}!", task_id);
                continue;
            }
            rprintln!("Running task {}", task_id);
            tasks[task_id].poll(task_id);
        }
        rprintln!("No tasks ready, going to sleep...");
        asm::wfi();
    }
}
