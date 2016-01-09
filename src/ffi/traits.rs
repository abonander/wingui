use winapi::*;

use std::ptr;

use winstr::WinString;

use super::{WindowHandle, RET_ERR};

pub trait WindowEvents: Sized {
    type Data: WindowData;

    fn on_create(_: &WindowHandle<Self>) {}

    fn on_show(_: &WindowHandle<Self>) {}

    fn on_hide(_: &WindowHandle<Self>) {}

    fn on_destroy(_: &WindowHandle<Self>) {}

    fn handle_msg(_: &WindowHandle<Self>, msg: DWORD) -> LRESULT { RET_ERR }
}

pub trait WindowData {
    fn name(&self) -> Option<&WinString> { None } 

    fn pos(&self) -> [c_int; 2] { [CW_USEDEFAULT, CW_USEDEFAULT] }

    fn size(&self) -> [c_int; 2] { [CW_USEDEFAULT, CW_USEDEFAULT] }

    fn parent(&self) -> HWND { ptr::null_mut() }

    fn menu(&self) -> HMENU { ptr::null_mut() }

    fn is_subclass(&self) -> bool { true }
}

pub trait BorrowHandle<W> where W: WindowEvents {
    fn borrow_handle(&self) -> WindowHandle<W>;
}
