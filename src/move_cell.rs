use std::cell::UnsafeCell;
use std::mem;

pub struct MoveCell<T> {
    inner: UnsafeCell<Option<T>>,
}

impl<T> MoveCell<T> {
    pub fn new() -> MoveCell<T> {
        Self::with_opt(None)   
    }

    pub fn with_val(val: T) -> MoveCell<T> {
        Self::with_opt(Some(val))
    }

    pub fn with_opt(opt: Option<T>) -> MoveCell<T> {
        MoveCell {
            inner: UnsafeCell::new(opt)
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
        self.take().map(|mut val| {
            let ret = mut_fn(&mut val);
            self.set(val);
            ret
        })
    }
}

impl<T: Clone> Clone for MoveCell<T> {
    fn clone(&self) -> MoveCell<T> {
        Self::with_opt(self.with_mut_ref(|val| val.clone()))
    }
}

impl<T> Default for MoveCell<T> {
    fn default() -> Self {
        Self::new()
    }
}
