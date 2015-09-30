use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

pub struct WinString {
    data: Vec<u16>,
}

impl WinString {
    pub fn new<S: AsRef<str>>(string: S) -> WinString {
        let string = string.as_ref();
        
        assert!(string.as_bytes().iter().all(|&byte| byte != 0), "String contains null bytes!");
               
        let mut data: Vec<_> = <str as AsRef<OsStr>>::as_ref(string).encode_wide().collect();
        data.push(0);
        
        WinString {
            data: data,
        }
    }

    pub fn as_ptr(&self) -> *const u16 {
        self.data.as_ptr()
    }
}
