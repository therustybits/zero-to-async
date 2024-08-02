use core::{
    cell::{Cell, RefCell},
    future::poll_fn,
    task::{Poll, Waker},
};

/// Storing the `Waker` directly this time, just to see how that works.
/// There is no more executor dependency, which is nice..
pub struct Channel<T> {
    item: Cell<Option<T>>,
    waker: RefCell<Option<Waker>>,
}

impl<T> Channel<T> {
    pub fn new() -> Self {
        Self {
            item: Cell::new(None),
            waker: RefCell::new(None),
        }
    }

    pub fn get_sender(&self) -> Sender<T> {
        Sender { channel: &self }
    }

    pub fn get_receiver(&self) -> Receiver<T> {
        Receiver { channel: &self }
    }

    fn send(&self, item: T) {
        self.item.replace(Some(item));
        if let Some(waker) = self.waker.borrow().as_ref() {
            // Calling `wake()` consumes the waker, which means we'd have to
            // `clone()` it first, so instead here we use `wake_by_ref()`
            waker.wake_by_ref();
        }
    }

    fn receive(&self, waker: Waker) -> Option<T> {
        if self.waker.borrow().as_ref().is_none() {
            self.waker.replace(Some(waker));
        }
        self.item.take()
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

pub struct Receiver<'a, T> {
    channel: &'a Channel<T>,
}

impl<T> Receiver<'_, T> {
    pub async fn receive(&mut self) -> T {
        poll_fn(|cx| match self.channel.receive(cx.waker().clone()) {
            Some(item) => Poll::Ready(item),
            None => Poll::Pending,
        })
        .await
    }
}
