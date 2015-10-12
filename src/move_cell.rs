use std::cell::UnsafeCell;
use std::mem;

#[derive(Default)]
pub struct MoveCell<T> {
    inner: UnsafeCell<Option<T>>,
}

impl<T> MoveCell<T> {
    pub fn new() -> MoveCell<T> {
        MoveCell {
            inner: UnsafeCell::new(None),
        }
    }

    pub fn with_val(val: T) -> MoveCell<T> {
        MoveCell {
            inner: UnsafeCell::new(Some(val)),
        }
    }

    pub unsafe fn mut_ref(&self) -> &mut Option<T> {
        &mut *self.inner.get()
    }

    pub fn take(&self) -> Option<T> {
        unsafe { self.mut_ref().take() }
    }

    pub fn set(&self, val: T) -> Option<T> {
        unsafe {
            mem::replace(self.mut_ref(), Some(val))
        }
    }

    pub fn is_set(&self) -> bool {
        unsafe { self.mut_ref().is_some() }
    }

    pub fn set_if_unset(&self, val: T) -> Option<T> {
        if self.is_set() {
            Some(val)
        } else {
            self.set(val)
        }
    }

    pub fn with_mut_ref<F, R>(&self, mut_fn: F) -> Option<R> where F: FnOnce(&mut T) -> R {
        let val = self.take();

        let ret = val.as_mut().map(mut_fn);

        self.set(val);

        ret
    }
}

impl<T: Clone> Clone for MoveCell<T> {
    fn clone(&self) -> MoveCell<T> {
        self.with_mut_ref(T::clone)
    }
}
