use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

pub struct WinString {
    data: Vec<u16>,
}

impl WinString {
    pub fn empty() -> WinString {
        WinString {
            data: vec![0],
        }
    }    

    pub fn from_str<S: AsRef<str>>(string: S) -> WinString {
        let mut winstr = Self::empty();
        winstr.replace(string);
        winstr
    }

    pub fn replace<S: AsRef<str>>(&mut self, string: S) {
        unsafe {
            self.data.set_len(0);
        }

        let string = string.as_ref();
        
        assert!(string.as_bytes().iter().all(|&byte| byte != 0), "String contains null bytes!");

        let string: &OsStr = string.as_ref();
               
        self.data.extend(string.encode_wide());
        self.data.push(0);
    } 

    pub fn as_ptr(&self) -> *const u16 {
        self.data.as_ptr()
    }
}
