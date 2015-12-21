#![feature(associated_type_defaults, catch_panic)]

extern crate winapi;
extern crate kernel32;
extern crate user32;

//mod abs_window;
//mod builder;
//mod context;
mod ffi;
//mod move_cell;
mod winstr;
//mod window;

//pub mod text;

use winstr::WinString;
//use window::{Window, WindowBuilder};

use std::any::Any;
use std::borrow::Cow;
use std::cell::UnsafeCell;
use std::error::Error;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::{mem, ptr, thread};

/*
pub fn start<F>(title: &str, window_fn: F) where F: FnOnce(WindowBuilder) -> WindowBuilder {
    unsafe {
        
        let mut msg = mem::zeroed();

        while user32::GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            user32::TranslateMessage(&msg);
            user32::DispatchMessageW(&msg);
        }
    }    
}
*/

fn last_error_code() -> winapi::DWORD {
    unsafe {
        kernel32::GetLastError()
    }
}

