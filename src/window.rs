use {kernel32, user32};

use winapi::*;

use ::{Context, ExnSafePtr};
use builder::{Builder, Buildable};
use winstr::WinString;

use std::marker::PhantomData;
use std::{mem, ptr};

static mut CLASS_ATOM: ATOM = 0;

const WINDOW_CLASS_NAME: &'static str = "WinGUIWindow";

unsafe extern "system" fn window_cb(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let ctxt = Context::mut_ref();

    let hwnd = ExnSafePtr(hwnd);    
    let data = ExnSafePtr(user32::GetWindowLongPtrW(hwnd.ptr(), GWLP_USERDATA) as *mut WindowData);

    match msg {
        WM_CREATE => {
            ctxt.catch_panic(move || {
                let on_create_cb = unsafe {
                    data.mut_ref().on_create.take()
                };

                let mut window = Window {
                   hwnd: hwnd.ptr(),
                   _marker: PhantomData,
                };

                on_create_cb.map(|mut f| f(&mut window))
            });

        },
        WM_CLOSE => { user32::DestroyWindow(hwnd.ptr()); },
        WM_DESTROY => { user32::PostQuitMessage(0); },
        _ => return user32::DefWindowProcW(hwnd.ptr(), msg, wparam, lparam),
    }

    0
}

unsafe fn register_window_class() -> ATOM {
    let ctxt = Context::mut_ref();

    let app_icon = user32::LoadIconW(ptr::null_mut(), IDI_APPLICATION);

    let class_name = ctxt.convert_str(WINDOW_CLASS_NAME);

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

pub struct Window<'a> {
    hwnd: HWND,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Buildable<'a> for Window<'a> {
    type Builder = WindowBuilder<'a>;
}

#[derive(Default)]
struct WindowData {
    on_create: Option<Box<FnMut(&mut Window)>>,
    on_show: Option<Box<FnMut(&mut Window)>>,
}

pub struct WindowBuilder<'a> {
    ctxt: &'a mut Context,
    title: &'a str,
    data: WindowData,
}

impl<'a> Builder<'a> for WindowBuilder<'a> {
    type InitArgs = &'a str;
    type Final = Window<'a>;

    fn new(ctxt: &'a mut Context, title: &'a str) -> Self {
        WindowBuilder {
            ctxt: ctxt,
            title: title,
            data: WindowData::default(),
        }
    }

    fn build(self) -> Window<'a> {
        let WindowBuilder { ctxt, title, data } = self;

        let title = ctxt.convert_str(title);
        
        let hwnd = unsafe {
            open_window(title, data)
        };

        Window { hwnd: hwnd, _marker: PhantomData }
    }
}

impl<'a> WindowBuilder<'a> {
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }

    pub fn on_create<F: FnOnce(&mut Window) + 'static>(mut self, on_create: F) -> Self {
        let mut on_create = Some(on_create);

        // This is a hack around the fact that `Box<FnOnce()>` cannot be directly invoked.
        self.data.on_create = Some(Box::new(
            move |win| if let Some(f) = on_create.take() { f(win); }
        ));

        self
    }

    pub fn on_show<F: FnMut(&mut Window) + 'static>(mut self, on_show: F) -> Self {
        self.data.on_show = Some(Box::new(on_show));
        self
    }
}

unsafe fn open_window(title: &WinString, data: WindowData) -> HWND {
    let class_atom = get_class_atom();

    let data = Box::new(data);

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

    user32::SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(data) as LONG_PTR);     

    user32::ShowWindow(hwnd, SW_SHOWNORMAL);
    user32::UpdateWindow(hwnd);

    hwnd
}

