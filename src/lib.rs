#![feature(const_fn, std_panic, recover)]

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;

extern crate winapi;
extern crate kernel32;
extern crate user32;

//mod abs_window;
//mod context;
mod ffi;
mod move_cell;
mod winstr;

pub mod window;
// pub mod text;

pub use move_cell::MoveCell;
use winstr::WinString;
//use window::{Window, WindowBuilder};

use std::any::Any;
use std::borrow::Cow;
use std::cell::UnsafeCell;
use std::error::Error;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::{mem, ptr, slice, thread};

use std::panic::{self, RecoverSafe};

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

        ffi::destroy_thread_windows();
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

fn last_err_msg() -> Result<String, winapi::DWORD> {
    use winapi::*;

    const FMT_FLAGS: DWORD = FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM;

    let mut buf_ptr: *const u16 = ptr::null_mut();

    let buf = unsafe {
        let len = kernel32::FormatMessageW(
            FMT_FLAGS, ptr::null(), last_error_code(), 0,
            &mut buf_ptr as *const _ as LPWSTR,
            0, ptr::null_mut()
        ) as usize;

        if len == 0 {
            return Err(last_error_code());
        }

        slice::from_raw_parts(buf_ptr, len)
    };

    let ret = String::from_utf16_lossy(buf);
    
    unsafe {
        if kernel32::HeapFree(kernel32::GetProcessHeap(), 0, buf_ptr as *mut _) == 0 {
            return Err(last_error_code());
        }
    }

    Ok(ret)
}

fn post_last_err_msg() {
    post_error(
        last_err_msg()
            .unwrap_or_else(|dword| format!("Win32 Error Code: {}", dword))
    )
}

fn post_error<E: Any + Send>(err: E) {
    post_error_boxed(Box::new(err));
}

fn post_error_boxed(err: Box<Any + Send>) {
    LAST_ERR.with(move |last| last.set(err));
}

pub fn quit() {
    unsafe {
        user32::PostQuitMessage(0);
    }
}

fn recover<F, R>(closure: F) -> Option<R> where F: FnOnce() -> R + RecoverSafe {
    match panic::recover(closure) {
        Ok(res) => Some(res),
        Err(err) => {
            post_error_boxed(err);
            None
        }
    }
}

