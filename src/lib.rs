#![feature(associated_type_defaults, catch_panic)]

#[macro_use] extern crate lazy_static;

extern crate winapi;
extern crate kernel32;
extern crate user32;

mod abs_window;
mod builder;
mod context;
mod move_cell;
mod winstr;
mod window;

pub mod text;

use winstr::WinString;
use window::{Window, WindowBuilder};

use std::any::Any;
use std::borrow::Cow;
use std::cell::UnsafeCell;
use std::error::Error;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::{mem, ptr, thread};

pub type WindowPtr = winapi::HWND;

pub use builder::{Builder, Buildable};
pub use context::{AbsContext, Context, CURRENT};

#[derive(Copy, Clone)]
struct ExnSafePtr<T>(*mut T);

impl<T> ExnSafePtr<T> {
    unsafe fn as_ref(&self) -> &T {
        & *self.0
    }

    fn ptr(&self) -> *mut T {
        self.0
    }
}

unsafe impl<T> Send for ExnSafePtr<T> {}

pub fn start<F>(title: &str, window_fn: F) where F: FnOnce(WindowBuilder) -> WindowBuilder {
    unsafe {
        CURRENT.with(|ctxt| {
            window_fn(Window::builder(&ctxt, title))
                .build();
        });

        let mut msg = mem::zeroed();

        while user32::GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            user32::TranslateMessage(&msg);
            user32::DispatchMessageW(&msg);
        }

        CURRENT.with(Context::reset);
    }    
}

unsafe fn win32_null_chk(ptr: WindowPtr) -> WindowPtr {
    assert!(!ptr.is_null(), "Windows operation failed! Error Code: {}", kernel32::GetLastError());
    ptr
}

