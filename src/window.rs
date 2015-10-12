use {kernel32, user32};

use winapi::*;

use ::{AbsContext, Context, ExnSafePtr, WindowPtr};
use abs_window::AbsWindow;
use builder::{Builder, Buildable};
use context::CURRENT;
use move_cell::MoveCell;
use winstr::WinString;

use std::marker::PhantomData;
use std::{mem, ptr};

static mut CLASS_ATOM: ATOM = 0;

const WINDOW_CLASS_NAME: &'static str = "WinGUIWindow";

unsafe extern "system" fn window_cb(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let hwnd = ExnSafePtr(hwnd);    
    let data = ExnSafePtr(user32::GetWindowLongPtrW(hwnd.ptr(), GWLP_USERDATA) as *mut WindowData);

    match msg {
        WM_CREATE => {
            CURRENT.with(|ctxt| {
                ctxt.catch_panic(move || {
                    let on_create_cb = unsafe {
                        data.as_ref().on_create.take()
                    };

                    let mut window = Window {
                       ptr: hwnd.ptr(),
                       _marker: PhantomData,
                    };

                    on_create_cb.map(|mut f| f(&mut window))
                });
            });
        },
        WM_CLOSE => { user32::DestroyWindow(hwnd.ptr()); },
        WM_DESTROY => { 
            user32::PostQuitMessage(0);
            let _ = Box::from_raw(data.ptr());
        },
        _ => return user32::DefWindowProcW(hwnd.ptr(), msg, wparam, lparam),
    }

    0
}

unsafe fn register_window_class() -> ATOM {
    let ctxt = Context::mut_ref();

    let app_icon = user32::LoadIconW(ptr::null_mut(), IDI_APPLICATION);

    let class_name = WinString::from_str(WINDOW_CLASS_NAME);

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

    let atom = user32::RegisterClassExW(&class_def);           

    if atom == 0 {
        panic!("Failed to register window class! Error code: {}", kernel32::GetLastError());
    }

    atom
}

unsafe fn get_class_atom() -> ATOM {
    if CLASS_ATOM == 0 {
        CLASS_ATOM = register_window_class();
    }

    CLASS_ATOM
}

pub struct Window {
    ptr: WindowPtr,
}

impl<'ctxt, 'init> Buildable<'ctxt, 'init> for Window<'ctxt> {
    type Builder = WindowBuilder<'ctxt, 'init>;

    fn builder(ctxt: &'ctxt Context, title: &'init str) -> WindowBuilder<'ctxt, 'init> {
        WindowBuilder {
            ctxt: ctxt,
            title: title,
            data: WindowData::default(),
        }
    }
}

unsafe impl<'ctxt> AbsContext for Window<'ctxt> {}

unsafe impl<'ctxt> AbsWindow for Window<'ctxt> {
    fn ptr(&self) -> WindowPtr { self.ptr }
}

#[derive(Default)]
struct WindowData {
    on_create: MoveCell<Box<FnMut(&Window)>>,
    on_show: MoveCell<Box<FnMut(&Window)>>,
}

pub struct WindowBuilder<'ctxt, 'init> {
    ctxt: &'ctxt Context,
    title: &'init str,
    data: WindowData,
}

impl<'ctxt, 'init> Builder<'ctxt, 'init> for WindowBuilder<'ctxt, 'init> {
    type Context = Context;
    type InitArgs = &'init str;
    type Final = Window<'ctxt>;

    fn build(self) -> Window<'ctxt> {
        let WindowBuilder { ctxt, title, data } = self;
        
        let hwnd = unsafe {
            open_window(title, data)
        };

        Window { ptr: hwnd, ctxt: ctxt }
    }
}

impl<'ctxt, 'init> WindowBuilder<'ctxt, 'init> {
    pub fn title(mut self, title: &'init str) -> Self {
        self.title = title;
        self
    }

    pub fn on_create<F: FnOnce(&Window) + 'static>(mut self, on_create: F) -> Self {
        let mut on_create = Some(on_create);

        // This is a hack around the fact that `Box<FnOnce()>` cannot be directly invoked.
        self.data.on_create.set(Box::new(
            move |win| if let Some(f) = on_create.take() { f(win); }
        ));

        self
    }

    pub fn on_show<F: FnMut(&Window) + 'static>(mut self, on_show: F) -> Self {
        self.data.on_show.set(Box::new(on_show));
        self
    }
}

unsafe fn open_window(title: &str, data: WindowData) -> WindowPtr {
    let class_atom = get_class_atom();

    let title = WinString::from_str(title);

    let data = Box::new(data);

    let hwnd = user32::CreateWindowExW(
        WS_EX_CLIENTEDGE,
        class_atom as *const u16,
        title.as_ptr(),
        WS_OVERLAPPEDWINDOW,
        CW_USEDEFAULT, CW_USEDEFAULT, 320, 240,
        ptr::null_mut(), ptr::null_mut(), ptr::null_mut(), ptr::null_mut(),
    );

    ::win32_null_chk(hwnd);

    user32::SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(data) as LONG_PTR);     

    user32::ShowWindow(hwnd, SW_SHOWNORMAL);
    user32::UpdateWindow(hwnd);

    hwnd
}

