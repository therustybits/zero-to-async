use core::cell::Cell;

use crate::{
    executor::wake_task,
    future::{OurFuture, Poll},
};

pub struct Channel<T> {
    item: Cell<Option<T>>,
    task_id: Cell<Option<usize>>,
}

impl<T> Channel<T> {
    pub fn new() -> Self {
        Self {
            item: Cell::new(None),
            task_id: Cell::new(None),
        }
    }

    pub fn get_sender(&self) -> Sender<T> {
        Sender { channel: &self }
    }

    pub fn get_receiver(&self) -> Receiver<T> {
        Receiver {
            channel: &self,
            state: ReceiverState::Init,
        }
    }

    fn send(&self, item: T) {
        self.item.replace(Some(item));
        if let Some(task_id) = self.task_id.get() {
            wake_task(task_id);
        }
    }

    fn receive(&self) -> Option<T> {
        self.item.take()
    }

    fn register(&self, task_id: usize) {
        self.task_id.replace(Some(task_id));
    }
}

pub struct Sender<'a, T> {
    channel: &'a Channel<T>,
}

impl<T> Sender<'_, T> {
    pub fn send(&self, item: T) {
        self.channel.send(item);
    }
}

enum ReceiverState {
    Init,
    Wait,
}

pub struct Receiver<'a, T> {
    channel: &'a Channel<T>,
    state: ReceiverState,
}

impl<T> OurFuture for Receiver<'_, T> {
    type Output = T;
    fn poll(&mut self, task_id: usize) -> Poll<Self::Output> {
        match self.state {
            ReceiverState::Init => {
                self.channel.register(task_id);
                self.state = ReceiverState::Wait;
                Poll::Pending
            }
            ReceiverState::Wait => match self.channel.receive() {
                Some(item) => Poll::Ready(item),
                None => Poll::Pending,
            },
        }
    }
}
