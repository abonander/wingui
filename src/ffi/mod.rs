use winapi::*;

use {kernel32, user32};

use winstr::WinString;

use std::marker::PhantomData;
use std::panic::AssertRecoverSafe;
use std::sync::RwLock;

use std::{mem, ptr, thread};

use self::class::Class;
use self::traits::{WindowEvents, WindowData};

pub mod class;
pub mod traits;

mod error;

pub use self::error::WindowsError;

pub type FFIResult<T> = Result<T, WindowsError>;

const RET_ERR: LRESULT = -1;

macro_rules! unwrap_or_ret (
    ($expr:expr, $or:expr) => (
        if let Some(val) = $expr {
            val
        } else {
            return $or;
        }
    )
);

pub struct WindowHandle<W: WindowEvents> {
    hwnd: HWND,
    data: *mut ProcData<<W as WindowEvents>::Data>,
    fresh: bool,
}

impl<W: WindowEvents> WindowHandle<W> {
    pub fn create_instance<C: Class>(class: C, data: <W as WindowEvents>::Data) -> FFIResult<Self> {
        let window_name = data.name().map_or_else(ptr::null, WinString::as_ptr);

        let pos = data.pos();
        let size = data.size();
        let parent = data.parent();
        let menu = data.menu();

        let hwnd = unsafe {
            user32::CreateWindowExW(
                WS_EX_CLIENTEDGE,
                class.atom(),
                window_name,
                WS_OVERLAPPEDWINDOW,
                pos[0], pos[1], size[0], size[1],
                parent,
                menu,
                ptr::null_mut(),
                ptr::null_mut(),
            )
        };

        if hwnd.is_null() {
            Err(WindowsError::last())
        } else {
            let orig_proc = if class.is_system() {
                unsafe { set_wnd_proc::<W>(hwnd) }
            } else {
                None
            };

            let proc_data = ProcData {
                orig_proc: orig_proc,
                window_data: data,
            };

            let data_ptr = Box::into_raw(Box::new(proc_data));

            let handle = WindowHandle {
                hwnd: hwnd,
                data: data_ptr,
                fresh: true,
            };

            // No recover here because this will be called by user code,
            // so panics should be allowed to propagate.
            W::on_create(&handle);

            Ok(handle)
        }
    }

    fn from_ptrs(hwnd: HWND, data: *mut ProcData<<W as WindowEvents>::Data>) -> Self {
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
        self.fresh = false;
        user32::DestroyWindow(self.hwnd);

        if !self.data.is_null() {
            Box::from_raw(self.data);
            self.data = ptr::null_mut();
        }
    }

    fn hwnd(&self) -> HWND {
        self.hwnd
    }

    pub unsafe fn data_mut(&self) -> &mut <W as WindowEvents>::Data {
        &mut (*self.data).window_data
    }

    unsafe fn orig_proc(&self) -> WindowProc {
        (*self.data).orig_proc()
    }
}        

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

pub unsafe fn destroy_thread_windows() -> BOOL {
    unsafe extern "system" fn destroy_thread_proc(hwnd: HWND, _: LPARAM) -> BOOL {
        user32::DestroyWindow(hwnd)
    }

    let thread_id = kernel32::GetCurrentThreadId();
    user32::EnumThreadWindows(thread_id, Some(destroy_thread_proc), 0)
}

type WindowProc = unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT;

unsafe fn set_wnd_proc<W: WindowEvents>(hwnd: HWND) -> Option<WindowProc> {
    mem::transmute(user32::SetWindowLongPtrW(hwnd, GWLP_WNDPROC, window_proc::<W> as LONG_PTR))
}

struct ProcData<D> {
    orig_proc: Option<WindowProc>,
    window_data: D,
}

impl<D> ProcData<D> {
    fn orig_proc(&self) -> WindowProc {
        self.orig_proc.unwrap_or(::user32::DefWindowProcW)
    }
}

unsafe extern "system" fn window_proc<W: WindowEvents>(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT { 
    let mut handle = unwrap_or_ret!(WindowHandle::from_hwnd(hwnd), RET_ERR);

    let mut handle = AssertRecoverSafe::new(&mut handle);
     
    let res = match msg {
        WM_SHOWWINDOW => {
            ::recover(||{
                if wparam != 0 {
                    W::on_show(&handle);
                } else {
                    W::on_hide(&handle);
                }
                
                0
            })
        },
        WM_DESTROY => {
            let mut res = Some(RET_ERR);

            if !thread::panicking() { 
                res = ::recover(|| { W::on_destroy(&handle); 0 });
            }

            handle.cleanup();
            res
        },
        _ => {
            let mut res = ::recover(|| W::handle_msg(&handle, msg));
            res
        },
    };

    (handle.orig_proc())(hwnd, msg, wparam, lparam);

    res.unwrap_or(RET_ERR)
}

