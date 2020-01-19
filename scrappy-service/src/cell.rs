//! Custom cell impl

use std::task::{Context, Poll};
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

pub(crate) struct Cell<T> {
    pub(crate) inner: Rc<RefCell<T>>,
}

impl<T> Clone for Cell<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Cell<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T> Cell<T> {
    pub(crate) fn new(inner: T) -> Self {
        Self {
            inner: Rc::new(RefCell::new(inner)),
        }
    }

    pub(crate) fn strong_count(&self) -> usize {
        Rc::strong_count(&self.inner)
    }
}

impl<T: 'static +  crate::Service> crate::Service for Cell<T> {
    type Request = T::Request;
    type Response = T::Response;
    type Error = T::Error;
    type Future = T::Future;

    fn poll_ready(self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.borrow_mut().poll_ready(cx)
    }

    fn call(self, req: Self::Request) -> Self::Future {
        self.inner.borrow_mut().call(req)
    }
}
