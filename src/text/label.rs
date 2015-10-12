use abs_window::AbsWindow;
use builder::{Builder, Buildable};
use window::Window;
use winstr::WinString;
use winstr::consts as winstr_consts;
use ::WindowPtr;

use super::TextWindow;

use user32;
use winapi::*;

use std::marker::PhantomData;
use std::ptr;

pub struct Label<'wind, 'ctxt> {
    ptr: WindowPtr,
    window: &'wind Window<'ctxt>,
}

unsafe impl<'wind, 'ctxt> AbsWindow for Label<'wind, 'ctxt> {
    fn ptr(&self) -> WindowPtr { self.ptr }
}

unsafe impl<'wind, 'ctxt> TextWindow for Label<'ctxt> {}

impl<'wind, 'ctxt, 'init> Buildable<'wind, 'init> for Label<'wind, 'ctxt> {
    type Builder = LabelBuilder<'wind, 'ctxt>;

    fn builder(window: &'wind mut Window<'ctxt>, init_args: ()) -> LabelBuilder<'wind, 'ctxt, 'init> {
        LabelBuilder {
            window: window,
            text: None,
        }
    }
}

pub struct LabelBuilder<'wind, 'ctxt, 'init> {
    window: &'wind Window<'ctxt>,
    text: Option<&'init str>,
}

impl<'wind, 'ctxt, 'init> LabelBuilder<'wind, 'ctxt, 'init> {
    pub fn text(mut self, text: &'init str) -> Self {
        self.text = Some(text);
        self
    }
}

impl<'wind, 'ctxt, 'init> Builder<'wind, 'init> for LabelBuilder<'wind, 'ctxt, 'init> {
    type Context = Window<'ctxt>;
    type InitArgs = ();
    type Final = Label<'wind, 'ctxt>;

    fn build(self) -> Label<'wind, 'ctxt> {
        let text = self.text.map(WinString::from_str);

        let text_ptr = text.as_ref().map_or(winstr_consts::EMPTY_STR, WinString::as_ptr);

        //FIXME: replace with winapi definition
        const SS_SIMPLE: u32 = 0x0B;

        let ptr = unsafe {
            let ptr = user32::CreateWindowExW(
                WS_EX_CLIENTEDGE, 
                winstr_consts::STATIC.as_ptr(),
                winstr_consts::EMPTY_STR,
                WS_CHILD | WS_VISIBLE | SS_SIMPLE,
                0, 0, 100, 100,
                self.window.ptr(),
                ptr::null_mut(), ptr::null_mut(), ptr::null_mut() 
            );

            ::win32_null_chk(ptr)               
        };

    
        Label {
            ptr: ptr,
            window: self.window,
        }
    }
}
