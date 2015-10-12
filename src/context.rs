use builder::{Buildable, Builder};
use move_cell::MoveCell;
use winstr::WinString;
use window::{WindowBuilder, Window};

use std::any::Any;
use std::borrow::Cow;
use std::cell::UnsafeCell;
use std::thread;

thread_local! {
    pub static CURRENT: UnsafeCell<Context> = UnsafeCell::new(Context::new())
}

pub unsafe trait AbsContext: Sized {
    fn build<'a, W>(
        &'a mut self, 
        args: <<W as Buildable<'a>>::Builder as Builder<'a>>::InitArgs
    ) -> <W as Buildable<'a>>::Builder 
    where W: Buildable<'a> + 'a, <W as Buildable<'a>>::Builder: Builder<'a, Context=Self> {
        <W as Buildable>::builder(self, args)
    }
}

pub struct Context {
    last_err: MoveCell<Cow<'static, str>>,
}

impl Context {
    fn new() -> Context {
        Context { 
            last_err: None, 
        }
    } 

    #[doc(hidden)]
    pub fn set_err(&self, err: Box<Any + Send + 'static>) {
        let cow = match err.downcast::<String>() {
            Ok(err_msg) => (&*err_msg).clone().into(),
            Err(err) => match err.downcast::<&'static str>() {
                Ok(err_msg) => (&*err_msg).clone().into(),
                Err(err) => format!("{:?}", err).into(),
            }
        };

        self.last_err.set(cow);
    }

    #[doc(hidden)]
    pub unsafe fn reset(&self) {
        if let Some(last_err) = self.last_err.take() {
            panic!("{}", last_err);
        }
    }

    #[doc(hidden)]
    pub fn catch_panic<R, F: FnOnce(&Context) -> R + Send + 'static>(&mut self, closure: F) -> Option<R> {
        let result = thread::catch_panic(move || CURRENT.with(closure));

        match result {
            Ok(ret) => Some(ret),
            Err(err) => {
                self.set_err(err);
                None
            },
        }
    }

    pub fn window<'a>(&'a self, title: &'a str) -> WindowBuilder<'a> {
        Window::builder(self, title)
    }
}

unsafe impl AbsContext for Context {}
