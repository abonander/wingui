use {kernel32, user32};

use winapi::*;

//use builder::{Builder, Buildable};
use ffi::{WindowHandle, WindowClass, WindowData};
use winstr::WinString;

use std::borrow::Cow;
use std::marker::PhantomData;
use std::{mem, ptr};

const WINDOW_CLASS_NAME: &'static str = "WinGUIWindow";

#[derive(Clone)]
pub struct Window {
    hnd: WindowHandle<Class>,
}

impl Window {
    pub fn new<T: Into<Cow<'static, str>>(title: ) -> Window {
        

        let hnd = WindowHandle::new_instance(


}

unsafe impl<'ctxt> AbsWindow for Window<'ctxt> {
    fn ptr(&self) -> WindowPtr { self.ptr }
}

#[derive(Default)]
struct Data {
    title: Cow<'static, str>,
    on_create: Option<FnBox(&mut Window)>,
    on_show: Option<Box<FnMut(&mut Window)>>,
}

impl WindowData for Data {

}

struct Class;

impl WindowClass for Class {
    type Data = Data;
}

