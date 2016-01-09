use {user32, kernel32};
use winapi::*;

use winstr::WinString;

use super::error::WindowsError;
use super::traits::WindowEvents;

use std::collections::HashMap;
use std::sync::RwLock;

use std::{mem, ptr};

pub trait Class {
    fn is_system(&self) -> bool { false }

    fn atom(&self) -> *const u16;
}

pub trait CustomClass {
    type Events: WindowEvents;

    fn name() -> &'static str; 
 
    unsafe fn icon() -> HICON { ptr::null_mut() }

    unsafe fn small_icon() -> HICON { ptr::null_mut() }

    unsafe fn cursor() -> HCURSOR { ptr::null_mut() }

    unsafe fn background_brush() -> HBRUSH { (COLOR_WINDOW + 1) as usize as *mut _ }

    unsafe fn menu_name() -> Option<&'static str> { None } 
}

impl<T: CustomClass> Class for T {
    fn atom(&self) -> *const u16 {
        class_atom::<Self>() as *const u16
    }
}

lazy_static!{
    static ref CUSTOM_CLASSES: RwLock<HashMap<&'static str, ATOM>> = RwLock::new(HashMap::new());
}

pub fn class_atom<W: CustomClass>() -> ATOM {
    {
        let classes = CUSTOM_CLASSES.read().unwrap();
        if let Some(&atom) = classes.get(W::name()) {
            return atom;
        }
    }

    let atom = {
        let mut classes = CUSTOM_CLASSES.write().unwrap();

        let atom = unsafe { register_class::<W>() };

        if atom != 0 {
            classes.insert(W::name(), atom);
        } else {
            error!(
                "Failed to register window class {:?}. Error message: {}", 
                W::name(), WindowsError::last()
            );     
        }

        atom
    };

    assert!(
        atom != 0,
        "Window class {:?} failed to initialize. Check the logs for more information.",
        W::name()
    );

    atom 
}

unsafe fn register_class<W: CustomClass>() -> ATOM {
    let class_name = WinString::from_str(W::name());

    let menu_name = W::menu_name().map(WinString::from_str);

    let class_def = WNDCLASSEXW {
        cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
        style: 0,
        lpfnWndProc: Some(super::window_proc::<<W as CustomClass>::Events>),
        cbClsExtra: 0,
        cbWndExtra: 0,
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

mod system {
    #[derive(Copy, Clone)]
    pub struct SystemClass(&'static [u16]);

    impl super::Class for SystemClass {
        fn is_system(&self) -> bool { true }

        fn atom(&self) -> *const u16 {
            self.0.as_ptr()
        }
    }

    include!("system_classes.rs");
}
