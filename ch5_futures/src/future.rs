/// Our simplified version of Rust's `core::future::Future` trait, used to get
/// a feel for the architecture of an async runtime.
pub trait OurFuture {
    type Output;
    fn poll(&mut self, task_id: usize) -> Poll<Self::Output>;
}

/// Same as `core::task::Poll`
/// Redefined here without all of the attribute clutter.
pub enum Poll<T> {
    Pending,
    Ready(T),
}
