use {kernel32, user32};

use winapi::*;

use winstr::WinString;

use std::{mem, ptr};

static mut CLASS_ATOM: ATOM = 0;

const WINDOW_CLASS_NAME: &'static str = "WinGUIWindow";

unsafe extern "system" fn window_cb(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            let create_struct = *(lparam as *mut CREATESTRUCTW);
            
            let data = &mut *(create_struct.lpCreateParams as *mut WindowData);    
            data.on_create.take().map(|mut f| f());  

            user32::SetWindowLongPtrW(hwnd, GWLP_USERDATA, data as *mut _ as LONG_PTR);
        },
        WM_CLOSE => { user32::DestroyWindow(hwnd); },
        WM_DESTROY => { user32::PostQuitMessage(0); },
        _ => return user32::DefWindowProcW(hwnd, msg, wparam, lparam),
    }

    0
}

unsafe fn register_window_class() -> ATOM {
    let app_icon = user32::LoadIconW(ptr::null_mut(), IDI_APPLICATION);

    let class_name = WinString::new(WINDOW_CLASS_NAME);

    let mut class_def = WNDCLASSEXW {
        cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
        style: 0,
        lpfnWndProc: Some(window_cb),
        cbClsExtra: 0,
        cbWndExtra: mem::size_of::<WindowData>() as i32,
        hInstance: ptr::null_mut(),
        hIcon: app_icon,
        hCursor: user32::LoadCursorW(ptr::null_mut(), IDC_ARROW),
        hbrBackground: (COLOR_WINDOW + 1) as usize as *mut _,
        lpszMenuName: ptr::null_mut(),
        lpszClassName: class_name.as_ptr(),
        hIconSm: app_icon,
    };

    user32::RegisterClassExW(&class_def)            
}

unsafe fn get_class_atom() -> ATOM {
    if CLASS_ATOM == 0 {
        CLASS_ATOM = register_window_class();
    }

    CLASS_ATOM
}

#[derive(Default)]
struct WindowData {
    on_create: Option<Box<FnMut()>>,
    on_show: Option<Box<FnMut()>>,
}

pub struct WindowBuilder<'a> {
    title: &'a str,
    data: WindowData,
}

impl<'a> WindowBuilder<'a> {
    pub fn new(title: &'a str) -> Self {
        WindowBuilder {
            title: title,
            data: WindowData::default(),
        }
    }

    pub fn on_create<F: FnOnce() + 'static>(mut self, on_create: F) -> Self {
        let mut on_create = Some(on_create);

        // This is a hack around the fact that `Box<FnOnce()>` cannot be directly invoked.
        self.data.on_create = Some(Box::new(
            move || if let Some(f) = on_create.take() { f(); }
        ));

        self
    }

    pub fn on_show<F: FnMut() + 'static>(mut self, on_show: F) -> Self {
        self.data.on_show = Some(Box::new(on_show));
        self
    }

    pub fn open(self) -> HWND {
        let WindowBuilder { title, data } = self;
        
        unsafe {
            open_window(title, data)
        }
    }
}

unsafe fn open_window(title: &str, data: WindowData) -> HWND {
    let class_atom = get_class_atom();

    let title = WinString::new(title);

    let data = Box::new(data);

    let hwnd = user32::CreateWindowExW(
        WS_EX_CLIENTEDGE,
        class_atom as *const u16,
        title.as_ptr(),
        WS_OVERLAPPEDWINDOW,
        CW_USEDEFAULT, CW_USEDEFAULT, 320, 240,
        ptr::null_mut(), ptr::null_mut(), ptr::null_mut(), 
        Box::into_raw(data) as *mut _,
    );

    if hwnd.is_null() {
        panic!("Failed to open window! Error Code: {}", kernel32::GetLastError());
    }

    user32::ShowWindow(hwnd, SW_SHOWNORMAL);
    user32::UpdateWindow(hwnd);

    hwnd
}

