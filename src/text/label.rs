use window::Window;
use winstr::WinString;
use winstr::consts as winstr_consts;
use ffi::{self, WindowData, WindowEvents, WindowHandle};
use ffi::class::system as system_classes;

use super::TextWindow;

use user32;
use winapi::*;

use std::marker::PhantomData;
use std::ptr;

pub struct Label {
    hand: WindowHandle<Self>,
}

impl Label {
    pub fn new<T: AsRef<str>>(text: T) -> Label {
        let data = LabelData::new(text);
        let hand = WindowHandle::create_instance(system_classes::STATIC, data).unwrap();

        Label {
            hand: hand
        }
    }
}

impl WindowEvents for Label {
    type Data = LabelData;
}

struct LabelData {
    text: WinString,
}

impl LabelData {
    fn new<T: AsRef<str>>(text: T) -> LabelData {
        LabelData {
            text: WinString::from_str(text)
        }
    }
}

impl WindowData for LabelData {
    fn name(&self) -> Option<&WinString> { 
        Some(&self.text)
    }
}
