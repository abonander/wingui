extern crate winapi;
extern crate kernel32;
extern crate user32;

mod winstr;
mod window;

use std::{mem, ptr};

pub fn show_window(title: &str) {
    unsafe {
        let hwnd = window::WindowBuilder::new(title)
            .on_create(|| println!("Window opened!"))
            .open();
    
        let mut msg = mem::zeroed();

        while user32::GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            user32::TranslateMessage(&msg);
            user32::DispatchMessageW(&msg);
        }
    }
}

