use {kernel32, user32};
use winapi::minwindef::{ATOM, WPARAM, LPARAM, LRESULT, UINT};
use winapi::windef::{HWND, };
use winapi::winuser::{
    COLOR_WINDOW,
    CW_USEDEFAULT,
    IDI_APPLICATION,
    IDC_ARROW,
    SW_SHOWNORMAL,
    WM_CLOSE, 
    WM_DESTROY, 
    WNDCLASSEXW,
    WS_EX_CLIENTEDGE,
    WS_OVERLAPPEDWINDOW,
};

use winstr::WinString;

use std::{mem, ptr};

static mut CLASS_ATOM: ATOM = 0;

const WINDOW_CLASS_NAME: &'static str = "KissUIWindow";

unsafe extern "system" fn window_cb(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
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

struct WindowData {
    on_click: Option<Box<FnMut()>>,
}


pub unsafe fn open_window(title: &str) -> HWND {
    let class_atom = get_class_atom();

    let title = WinString::new(title);

    let hwnd = user32::CreateWindowExW(
        WS_EX_CLIENTEDGE,
        class_atom as *const u16,
        title.as_ptr(),
        WS_OVERLAPPEDWINDOW,
        CW_USEDEFAULT, CW_USEDEFAULT, 320, 240,
        ptr::null_mut(), ptr::null_mut(), ptr::null_mut(), ptr::null_mut(),
    );

    if hwnd.is_null() {
        panic!("Failed to open window! Error Code: {}", kernel32::GetLastError());
    }

    user32::ShowWindow(hwnd, SW_SHOWNORMAL);
    user32::UpdateWindow(hwnd);

    hwnd
}



