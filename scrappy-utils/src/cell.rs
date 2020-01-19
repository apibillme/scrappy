//! Custom cell impl

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

    pub(crate) fn get_mut(self) -> &'static mut T {
        self.inner.get_mut()
    }
}
