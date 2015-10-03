#![feature(catch_panic)]

extern crate winapi;
extern crate kernel32;
extern crate user32;

mod builder;
mod winstr;
mod window;

use builder::Builder;
use winstr::WinString;
use window::WindowBuilder;

use std::any::Any;
use std::borrow::Cow;
use std::cell::UnsafeCell;
use std::error::Error;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::{mem, ptr, thread};

thread_local! {
    static CURRENT_CTXT: UnsafeCell<Context> = UnsafeCell::new(Context::new())
}

pub struct Context {
    con_buf: WinString,
    last_err: Option<Cow<'static, str>>,
}

impl Context {
    fn new() -> Context {
        Context { 
            con_buf: WinString::empty(), 
            last_err: None, 
        }
    }

    unsafe fn mut_ref<'a>() -> &'a mut Self {
        let ctxt_ptr = CURRENT_CTXT.with(|ctxt| ctxt.get());
        &mut *ctxt_ptr
    }

    /// Convert a Rust string to UTF-16.
    /// 
    /// The returned pointer should be valid until the next time this is called. 
    fn convert_str<S: AsRef<str>>(&mut self, string: S) -> &WinString {        
        self.con_buf.replace(string);
        &self.con_buf
    }

    fn set_err(&mut self, err: Box<Any + Send + 'static>) {
        let cow = match err.downcast::<String>() {
            Ok(err_msg) => (&*err_msg).clone().into(),
            Err(err) => match err.downcast::<&'static str>() {
                Ok(err_msg) => (&*err_msg).clone().into(),
                Err(err) => format!("{:?}", err).into(),
            }
        };

        self.last_err = Some(cow);
    }

    fn reset(&mut self) {
        let last_err = self.last_err.take();

        if let Some(last_err) = last_err {
            panic!("{}", last_err);
        }
    }

    fn catch_panic<R, F: FnOnce() -> R + Send + 'static>(&mut self, closure: F) -> Option<R> {
        match thread::catch_panic(closure) {
            Ok(ret) => Some(ret),
            Err(err) => {
                self.set_err(err);
                None
            },
        }
    }


    pub fn window<'a>(&'a mut self, title: &'a str) -> WindowBuilder<'a> {
        WindowBuilder::new(self, title)
    }
}

#[derive(Copy, Clone)]
struct ExnSafePtr<T>(*mut T);

impl<T> ExnSafePtr<T> {
    unsafe fn mut_ref(&self) -> &mut T {
        &mut *self.0
    }

    fn ptr(&self) -> *mut T {
        self.0
    }
}

unsafe impl<T> Send for ExnSafePtr<T> {}

pub fn start<F>(title: &str, window_fn: F) 
where F: for<'a> FnOnce(WindowBuilder<'a>) -> WindowBuilder<'a> {
    unsafe {
        {
            let mut ctxt = Context::mut_ref();         
            window_fn(WindowBuilder::new(&mut ctxt, title))
                .build();
        }

        let mut msg = mem::zeroed();

        while user32::GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            user32::TranslateMessage(&msg);
            user32::DispatchMessageW(&msg);
        }

        Context::mut_ref().reset()
    }    
}

