use winapi::*;

use {kernel32, user32};

use winstr::WinString;

use std::marker::PhantomData;
use std::sync::RwLock;

use std::{mem, panic, ptr};

mod class;

const RET_ERR: LRESULT = -1;

unsafe extern "system" fn window_proc<W: WindowEvents>(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let mut handle;
    
    if msg == WM_CREATE {
        let data = (*(lparam as *mut CREATESTRUCTW)).lpCreateParams as *mut _;
        handle = WindowHandle::from_ptrs(hwnd, data);
        user32::SetWindowLongPtrW(hwnd, GWLP_USERDATA, data as i64);
    } else {
        if let Some(handle_) = WindowHandle::from_hwnd(hwnd) {
            handle = handle_;
        } else {
            return RET_ERR;
        }
    }
     
    let res = match msg { 
        WM_CREATE => ::recover(|| { W::on_create(&handle); 0 }),
        WM_DESTROY => {
            let res = ::recover(|| { W::on_destroy(&handle); 0 });
            handle.cleanup();
            res
        },
        _ => {
            let mut res = ::recover(|| W::handle_msg(&handle, msg));

            if res == Some(RET_ERR) {
                res = Some(::user32::DefWindowProcW(hwnd, msg, wparam, lparam));
            }

            res
        },
    };

    res.unwrap_or(RET_ERR)
}

pub struct WindowHandle<W: WindowEvents> {
    hwnd: HWND,
    data: *mut <W as WindowEvents>::Data,
    fresh: bool,
}

impl<W: WindowEvents> WindowHandle<W> {
    fn create_instance(class_atom: DWORD, data: <W as WindowEvents>::Data) -> Result<Self, DWORD> {
        let window_name = data.name().map_or_else(ptr::null, WinString::as_ptr);

        let pos = data.pos();
        let size = data.size();
        let parent = data.parent();
        let menu = data.menu();

        let data_ptr = Box::into_raw(Box::new(data));

        let hwnd = unsafe {
            user32::CreateWindowExW(
                WS_EX_CLIENTEDGE,
                class_atom as *const u16,
                window_name,
                WS_OVERLAPPEDWINDOW,
                pos[0], pos[1], size[0], size[1],
                parent,
                menu,
                ptr::null_mut(),
                data_ptr as LPVOID,
            )
        };

        if hwnd.is_null() {
            Err(::last_error_code())
        } else {
            Ok(WindowHandle {
                hwnd: hwnd,
                data: data_ptr,
                fresh: true,
            })
        }
    }

    fn from_ptrs(hwnd: HWND, data: *mut <W as WindowEvents>::Data) -> Self {
        WindowHandle {
            hwnd: hwnd,
            data: data,
            fresh: false,
        }
    }

    unsafe fn from_hwnd(hwnd: HWND) -> Option<Self> {
        let data_ptr = user32::GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut _;

        if data_ptr.is_null() {
            None
        } else {
            Some(Self::from_ptrs(hwnd, data_ptr))
        }
    } 
    
    unsafe fn cleanup(&mut self) {
        user32::DestroyWindow(self.hwnd);
        Box::from_raw(self.data);
        self.fresh = false;
    }

    fn hwnd(&self) -> HWND {
        self.hwnd
    }

    unsafe fn data_mut(&self) -> &mut <W as WindowEvents>::Data {
        &mut *self.data
    }
}        

impl<W: WindowEvents> panic::NoUnsafeCell for WindowHandle<W> {}

impl<W: WindowEvents> Clone for WindowHandle<W> {
    fn clone(&self) -> Self {
        WindowHandle {
            hwnd: self.hwnd,
            data: self.data,
            fresh: false,
        }
    }
}

impl<W: WindowEvents> Drop for WindowHandle<W> {
    fn drop(&mut self) {
        if self.fresh {
            unsafe {
                self.cleanup();
            }
        }
    }
}

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
