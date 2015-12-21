use winapi::*;

use {kernel32, user32};

use winstr::WinString;

use std::marker::PhantomData;
use std::sync::Once;

use std::{mem, panic, ptr};

const RET_ERR: LRESULT = -1;

unsafe extern "system" fn window_proc<W: WindowClass>(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
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
        _ => ::recover(|| W::handle_msg(&handle, msg)),
    };

    res.unwrap_or(RET_ERR)
}

fn class_atom<W: WindowClass>() -> ATOM {
        static CLASS_INIT: Once = Once::new();
        static mut CLASS_ATOM: ATOM = 0;

        CLASS_INIT.call_once(|| unsafe { 
            CLASS_ATOM = register_class::<W>();
               
            if CLASS_ATOM == 0 {
                error!(
                    "Failed to register window class {:?}. Error code: {:X}", 
                       W::name(), ::last_error_code()
                );
            }
        });
                
        let atom = unsafe { CLASS_ATOM };
 
        assert!(
            atom != 0,
            "Window class {:?} failed to initialize. Check the logs for more information.",
            W::name()
        );

        atom 
    }

unsafe fn register_class<W: WindowClass>() -> ATOM {
    let class_name = WinString::from_str(W::name());

    let menu_name = W::menu_name().map(WinString::from_str);

    let class_def = WNDCLASSEXW {
        cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
        style: 0,
        lpfnWndProc: Some(window_proc::<W>),
        cbClsExtra: 0,
        cbWndExtra: mem::size_of::<*mut <W as WindowClass>::Data>() as i32,
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


pub struct WindowHandle<W: WindowClass> {
    hwnd: HWND,
    data: *mut <W as WindowClass>::Data,
    fresh: bool,
}

impl<W: WindowClass> WindowHandle<W> {
    fn create_instance(data: <W as WindowClass>::Data) -> Result<Self, DWORD> {
        let window_name = data.name().map_or_else(ptr::null, WinString::as_ptr);
        let class_atom = class_atom::<W>();

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
            unsafe {
                user32::ShowWindow(hwnd, SW_SHOWNORMAL);
                user32::UpdateWindow(hwnd);
            }

            Ok(WindowHandle {
                hwnd: hwnd,
                data: data_ptr,
                fresh: true,
            })
        }
    }

    fn from_ptrs(hwnd: HWND, data: *mut <W as WindowClass>::Data) -> Self {
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

    unsafe fn data_mut(&self) -> &mut <W as WindowClass>::Data {
        &mut *self.data
    }
}        

impl<W: WindowClass> panic::RecoverSafe for WindowHandle<W> {}

impl<W: WindowClass> Clone for WindowHandle<W> {
    fn clone(&self) -> Self {
        WindowHandle {
            hwnd: self.hwnd,
            data: self.data,
            fresh: false,
        }
    }
}

impl<W: WindowClass> Drop for WindowHandle<W> {
    fn drop(&mut self) {
        if self.fresh {
            unsafe {
                self.cleanup();
            }
        }
    }
}

pub trait WindowClass: Sized {
    type Data: WindowData;

    fn name() -> &'static str; 
 
    unsafe fn icon() -> HICON { ptr::null_mut() }

    unsafe fn small_icon() -> HICON { ptr::null_mut() }

    unsafe fn cursor() -> HCURSOR { ptr::null_mut() }

    unsafe fn background_brush() -> HBRUSH { (COLOR_WINDOW + 1) as usize as *mut _ }

    unsafe fn menu_name() -> Option<&'static str> { None }

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
}
