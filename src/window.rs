use {kernel32, user32};

use winapi::*;

//use builder::{Builder, Buildable};
use ffi::WindowHandle;
use ffi::class::{Class as WindowClass, CustomClass};
use ffi::traits::{BorrowHandle, WindowEvents, WindowData};
use winstr::WinString;

use std::borrow::Cow;
use std::boxed::FnBox;
use std::marker::PhantomData;
use std::{mem, ptr};

#[derive(Clone)]
pub struct Window {
    hnd: WindowHandle<Class>,
}

impl Window {
    pub fn new<T: AsRef<str>>(title: T) -> Window {
        let data = Data::new(title);
        let hnd = WindowHandle::create_instance(Class, data).unwrap();

        Window {
            hnd: hnd
        }
    }

    pub fn add_child<W: WindowEvents, C: BorrowHandle<W>>(&mut self, child: C) -> &mut Self {

        self
    }
}

#[derive(Default)]
struct Data {
    title: WinString,
    on_create: Option<Box<FnMut(&mut Window)>>,
    on_show: Option<Box<FnMut(&mut Window)>>,
}

impl Data {
    fn new<T: AsRef<str>>(title: T) -> Data {
        Data { title: WinString::from_str(title), .. Data::default() }
    }
}

impl WindowData for Data {
    fn name(&self) -> Option<&WinString> {
        Some(&self.title)
    }
}

struct Class;

impl WindowEvents for Class {
    type Data = Data;

    fn on_create(hnd: &WindowHandle<Self>) {
        let cb = unsafe {
            hnd.data_mut().on_create.take()
        };
        let mut wnd = Window { hnd: hnd.clone() };
        
        cb.map(|mut on_create| on_create(&mut wnd));
    }

    fn on_show(hnd: &WindowHandle<Self>) {
        let cb = unsafe {
            hnd.data_mut().on_show.as_mut()
        };
        
        let mut wnd = Window { hnd: hnd.clone() };
        
        cb.map(|on_show| (on_show)(&mut wnd));
    }
}

impl CustomClass for Class {
    type Events = Self;

    fn name() -> &'static str {
        "WinGUI Main Window"
    }
}
