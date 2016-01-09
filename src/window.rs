use {kernel32, user32};

use winapi::*;

//use builder::{Builder, Buildable};
use ffi::{WindowHandle, WindowClass, WindowData};
use ffi::class::{Class, CustomClass};
use winstr::WinString;

use std::borrow::Cow;
use std::marker::PhantomData;
use std::{mem, ptr};

#[derive(Clone)]
pub struct Window {
    hnd: WindowHandle<Class>,
}

impl Window {
    pub fn new<T: AsRef<str>>(title: T) -> Window {
        let data = Data::new(title);
        let hnd = WindowHandle::new_instance(Class, data);

        Window {
            hnd: hnd
        }
    }

    pub fn add_child<W: WindowEvents, C: BorrowHandle<W>>(&mut self, child: C) -> &mut Self {

    }
}

#[derive(Default)]
struct Data {
    title: WinString,
    on_create: Option<FnBox(&mut Window)>,
    on_show: Option<Box<FnMut(&mut Window)>>,
}

impl Data {
    fn new<T: AsRef<str>>(title: T) -> Data {
        Data { title: WinString::from_str(title), ... Data::default() }
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
        unsafe {
            self.data_mut().on_create.take()
        }.map(|on_create| (on_create)());
    }

    fn on_show(hnd: &WindowHandle<Self>) {
        unsafe {
            self.data_mut().on_show.as_mut()
        }.map(|on_show| (on_show)())
    }
}

impl CustomClass for Class {
    type Events = Self;

    fn name() -> &'static str {
        "WinGUI Main Window"
    }
}
