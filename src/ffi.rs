use winapi::*;

use {kernel32, user32};

use winstr::WinString;

use std::marker::PhantomData;
use std::sync::atomic::{Ordering, AtomicIsize, ATOMIC_ISIZE_INIT};

use std::{mem, ptr};

unsafe extern "system" fn window_proc<W: WindowClass>(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {

    0
}


pub struct WindowHandle<W: WindowClass> {
    ptr: HWND,
    _marker: PhantomData<W>,
}

impl<W: WindowClass> WindowHandle<W> {
    fn class_atom() -> ATOM {
        static CLASS_ATOM: AtomicIsize = ATOMIC_ISIZE_INIT;

        let atom = CLASS_ATOM.load(Ordering::Acquire);

        // Fast path: class already registered.
        if atom > 0 {
            return atom as ATOM;
        }

        // Attempt to swap in the sentinel value
        match CLASS_ATOM.compare_and_swap(0, -1, Ordering::AcqRel) {
            // We're the first; register the class and replace the sentinel value.
            0 => {
                let atom = unsafe { Self::register_class() };
               
                if atom == 0 {
                    CLASS_ATOM.store(0, Ordering::Release);

                    panic!("Failed to register window class. Error code: {}", ::last_error_code());
                }
                
                CLASS_ATOM.store(atom as isize, Ordering::Release);
                atom
            },
            // Another thread got to it first; wait until the class has been registered.
            -1 => {
                let mut atom = -1;

                while atom == -1 {
                    atom = CLASS_ATOM.load(Ordering::Acquire);
                }

                if atom == 0 {
                    panic!("Another thread panicked attempting to register this class.");
                }

                atom as ATOM
            },
            atom => atom as ATOM,
        }
    }

    unsafe fn register_class() -> ATOM {
        let class_name = WinString::from_str(W::class_name());

        let menu_name = W::menu_name().map(WinString::from_str);

        let class_def = WNDCLASSEXW {
            cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfnWndProc: Some(window_proc::<W>),
            cbClsExtra: 0,
            cbWndExtra: mem::size_of::<*mut W>() as i32,
            hInstance: ptr::null_mut(),
            hIcon: W::icon(),
            hCursor: W::cursor(),
            hbrBackground: W::background_brush(),
            lpszMenuName: menu_name.as_ref().map_or_else(ptr::null, WinString::as_ptr),
            lpszClassName: class_name.as_ptr(),
            hIconSm: W::small_icon(),
        };

        user32::RegisterClassExW(&class_def)        
    }
}        

pub trait WindowClass {
    fn class_name() -> &'static str;

    unsafe fn icon() -> HICON { ptr::null_mut() }

    unsafe fn small_icon() -> HICON { ptr::null_mut() }

    unsafe fn cursor() -> HCURSOR { ptr::null_mut() }

    unsafe fn background_brush() -> HBRUSH { (COLOR_WINDOW + 1) as usize as *mut _ }

    unsafe fn menu_name() -> Option<&'static str> { None }

}




