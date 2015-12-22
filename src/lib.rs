#![feature(const_fn, std_panic, recover)]

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;

extern crate winapi;
extern crate kernel32;
extern crate user32;

//mod abs_window;
//mod builder;
//mod context;
mod ffi;
mod move_cell;
mod winstr;
//mod window;

//pub mod text;

pub use move_cell::MoveCell;
use winstr::WinString;
//use window::{Window, WindowBuilder};

use std::any::Any;
use std::borrow::Cow;
use std::cell::UnsafeCell;
use std::error::Error;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::{mem, ptr, thread};

use std::panic::{self, RecoverSafe};


pub type Window = ();

thread_local!(static LAST_ERR: MoveCell<Box<Any + Send>> = MoveCell::new());

pub fn start<F>(init_fn: F) where F: FnOnce() -> Window {
    let window = init_fn();

    // window.show();

    mem::forget(window);
    
    unsafe { 
        let mut msg = mem::zeroed();

        while user32::GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            user32::TranslateMessage(&msg);
            user32::DispatchMessageW(&msg);
        }
    }

    if let Some(err) = LAST_ERR.with(|last| last.take()) {
        panic!(err);
    }
}

fn last_error_code() -> winapi::DWORD {
    unsafe {
        kernel32::GetLastError()
    }
}

fn recover<F, R>(closure: F) -> Option<R> where F: FnOnce() -> R + RecoverSafe {
    match panic::recover(closure) {
        Ok(res) => Some(res),
        Err(err) => {
            LAST_ERR.with(move |last| last.set(err));
            None
        }
    }
}

