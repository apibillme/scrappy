//! A one-shot, futures-aware channel.
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub use futures::channel::oneshot::Canceled;
use slab::Slab;

use crate::cell::Cell;
use crate::task::LocalWaker;

/// Creates a new futures-aware, one-shot channel.
pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Cell::new(Inner {
        value: None,
        rx_task: LocalWaker::new(),
    });
    let tx = Sender {
        inner: inner.clone(),
    };
    let rx = Receiver { inner };
    (tx, rx)
}

/// Creates a new futures-aware, pool of one-shot's.
pub fn pool<T>() -> Pool<T> {
    Pool(Cell::new(Slab::new()))
}

/// Represents the completion half of a oneshot through which the result of a
/// computation is signaled.
#[derive(Debug)]
pub struct Sender<T> {
    inner: Cell<Inner<T>>,
}

/// A future representing the completion of a computation happening elsewhere in
/// memory.
#[derive(Debug)]
#[must_use = "futures do nothing unless polled"]
pub struct Receiver<T> {
    inner: Cell<Inner<T>>,
}

// The channels do not ever project Pin to the inner T
impl<T> Unpin for Receiver<T> {}
impl<T> Unpin for Sender<T> {}

#[derive(Debug)]
struct Inner<T> {
    value: Option<T>,
    rx_task: LocalWaker,
}

impl<T: 'static> Sender<T> {
    /// Completes this oneshot with a successful result.
    ///
    /// This function will consume `self` and indicate to the other end, the
    /// `Receiver`, that the error provided is the result of the computation this
    /// represents.
    ///
    /// If the value is successfully enqueued for the remote end to receive,
    /// then `Ok(())` is returned. If the receiving end was dropped before
    /// this function was called, however, then `Err` is returned with the value
    /// provided.
    pub fn send(self, val: T) -> Result<(), T> {
        if self.inner.strong_count() == 2 {
            let mut inner = self.inner.inner.borrow_mut();
            inner.value = Some(val);
            inner.rx_task.clone().wake();
            Ok(())
        } else {
            Err(val)
        }
    }

    /// Tests to see whether this `Sender`'s corresponding `Receiver`
    /// has gone away.
    pub fn is_canceled(&self) -> bool {
        self.inner.strong_count() == 1
    }
}

struct SenderWrapper<T: 'static> {
    t: Sender<T>
}

impl<T: 'static> Drop for SenderWrapper<T> {
    fn drop(&mut self) {
        self.t.inner.inner.borrow_mut().rx_task.clone().wake();
    }
}

impl<T: 'static> Future for Receiver<T> {
    type Output = Result<T, Canceled>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        // If we've got a value, then skip the logic below as we're done.
        if let Some(val) = this.inner.inner.borrow_mut().value.take() {
            return Poll::Ready(Ok(val));
        }

        // Check if sender is dropped and return error if it is.
        this.inner.inner.borrow_mut().rx_task.clone().register(cx.waker());
        Poll::Pending
    }
}

/// Futures-aware, pool of one-shot's.
pub struct Pool<T>(Cell<Slab<PoolInner<T>>>);

bitflags::bitflags! {
    pub struct Flags: u8 {
        const SENDER = 0b0000_0001;
        const RECEIVER = 0b0000_0010;
    }
}

#[derive(Debug)]
struct PoolInner<T> {
    flags: Flags,
    value: Option<T>,
    waker: LocalWaker,
}

impl<T: 'static> Pool<T> {
    pub fn channel(&mut self) -> (PSender<T>, PReceiver<T>) {
        let token = self.0.inner.borrow_mut().insert(PoolInner {
            flags: Flags::all(),
            value: None,
            waker: LocalWaker::default(),
        });

        (
            PSender {
                token,
                inner: self.0.clone(),
            },
            PReceiver {
                token,
                inner: self.0.clone(),
            },
        )
    }
}

impl<T> Clone for Pool<T> {
    fn clone(&self) -> Self {
        Pool(self.0.clone())
    }
}

/// Represents the completion half of a oneshot through which the result of a
/// computation is signaled.
#[derive(Debug)]
pub struct PSender<T> {
    token: usize,
    inner: Cell<Slab<PoolInner<T>>>,
}

/// A future representing the completion of a computation happening elsewhere in
/// memory.
#[derive(Debug)]
#[must_use = "futures do nothing unless polled"]
pub struct PReceiver<T> {
    token: usize,
    inner: Cell<Slab<PoolInner<T>>>,
}

// The oneshots do not ever project Pin to the inner T
impl<T> Unpin for PReceiver<T> {}
impl<T> Unpin for PSender<T> {}

impl<T: 'static> PSender<T> {
    /// Completes this oneshot with a successful result.
    ///
    /// This function will consume `self` and indicate to the other end, the
    /// `Receiver`, that the error provided is the result of the computation this
    /// represents.
    ///
    /// If the value is successfully enqueued for the remote end to receive,
    /// then `Ok(())` is returned. If the receiving end was dropped before
    /// this function was called, however, then `Err` is returned with the value
    /// provided.
    pub fn send(self, val: T) -> Result<(), T> {
        let mut i = self.inner.inner.borrow_mut();
        let inner = i.get_mut(self.token).unwrap();

        if inner.flags.contains(Flags::RECEIVER) {
            inner.value = Some(val);
            inner.waker.clone().wake();
            Ok(())
        } else {
            Err(val)
        }
    }

    /// Tests to see whether this `Sender`'s corresponding `Receiver`
    /// has gone away.
    pub fn is_canceled(&self) -> bool {
        !self.inner.inner.borrow_mut().get_mut(self.token).unwrap()
            .flags
            .contains(Flags::RECEIVER)
    }
}

struct PSenderWrapper<T: 'static> {
    t: PSender<T>
}

impl<T: 'static> Drop for PSenderWrapper<T> {
    fn drop(&mut self) {
        let mut i = self.t.inner.inner.borrow_mut();
        let inner = i.get_mut(self.t.token).unwrap();
        if inner.flags.contains(Flags::RECEIVER) {
            inner.waker.clone().wake();
            inner.flags.remove(Flags::SENDER);
        } else {
            self.t.inner.inner.borrow_mut().remove(self.t.token);
        }
    }
}

struct PReceiverWrapper<T: 'static> {
    t: PReceiver<T>
}

impl<T: 'static> Drop for PReceiverWrapper<T> {
    fn drop(&mut self) {
        let mut i = self.t.inner.inner.borrow_mut();
        let inner = i.get_mut(self.t.token).unwrap();
        if inner.flags.contains(Flags::SENDER) {
            inner.flags.remove(Flags::RECEIVER);
        } else {
            self.t.inner.inner.borrow_mut().remove(self.t.token);
        }
    }
}

impl<T: 'static> Future for PReceiver<T> {
    type Output = Result<T, Canceled>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut i = self.inner.inner.borrow_mut();
        let inner = i.get_mut(self.token).unwrap();

        // If we've got a value, then skip the logic below as we're done.
        if let Some(val) = inner.value.take() {
            return Poll::Ready(Ok(val));
        }

        // Check if sender is dropped and return error if it is.
        if !inner.flags.contains(Flags::SENDER) {
            Poll::Ready(Err(Canceled))
        } else {
            inner.waker.clone().register(cx.waker());
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::future::lazy;

    #[scrappy_rt::test]
    async fn test_oneshot() {
        let (tx, rx) = channel();
        tx.send("test").unwrap();
        assert_eq!(rx.await.unwrap(), "test");

        let (tx, rx) = channel();
        assert!(!tx.is_canceled());
        drop(rx);
        assert!(tx.is_canceled());
        assert!(tx.send("test").is_err());

        let (tx, rx) = channel::<&'static str>();
        drop(tx);
        assert!(rx.await.is_err());

        let (tx, mut rx) = channel::<&'static str>();
        assert_eq!(lazy(|cx| Pin::new(&mut rx).poll(cx)).await, Poll::Pending);
        tx.send("test").unwrap();
        assert_eq!(rx.await.unwrap(), "test");

        let (tx, mut rx) = channel::<&'static str>();
        assert_eq!(lazy(|cx| Pin::new(&mut rx).poll(cx)).await, Poll::Pending);
        drop(tx);
        assert!(rx.await.is_err());
    }

    #[scrappy_rt::test]
    async fn test_pool() {
        let (tx, rx) = pool().channel();
        tx.send("test").unwrap();
        assert_eq!(rx.await.unwrap(), "test");

        let (tx, rx) = pool().channel();
        assert!(!tx.is_canceled());
        drop(rx);
        assert!(tx.is_canceled());
        assert!(tx.send("test").is_err());

        let (tx, rx) = pool::<&'static str>().channel();
        drop(tx);
        assert!(rx.await.is_err());

        let (tx, mut rx) = pool::<&'static str>().channel();
        assert_eq!(lazy(|cx| Pin::new(&mut rx).poll(cx)).await, Poll::Pending);
        tx.send("test").unwrap();
        assert_eq!(rx.await.unwrap(), "test");

        let (tx, mut rx) = pool::<&'static str>().channel();
        assert_eq!(lazy(|cx| Pin::new(&mut rx).poll(cx)).await, Poll::Pending);
        drop(tx);
        assert!(rx.await.is_err());
    }
}
