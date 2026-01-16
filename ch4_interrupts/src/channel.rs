use core::cell::Cell;

pub struct Channel<T> {
    item: Cell<Option<T>>,
}

impl<T> Channel<T> {
    pub fn new() -> Self {
        Self {
            item: Cell::new(None),
        }
    }

    pub fn get_sender(&self) -> Sender<'_, T> {
        Sender { channel: self }
    }

    pub fn get_receiver(&self) -> Receiver<'_, T> {
        Receiver { channel: self }
    }

    fn send(&self, item: T) {
        self.item.replace(Some(item));
    }

    fn receive(&self) -> Option<T> {
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
    pub fn receive(&self) -> Option<T> {
        self.channel.receive()
    }
}
