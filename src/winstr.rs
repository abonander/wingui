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

        let os_str: &OsStr = string.as_ref().as_ref();

        self.data.extend(os_str.encode_wide());
        self.data.push(0);
    } 

    pub fn as_ptr(&self) -> *const u16 {
        self.data.as_ptr()
    }

    pub unsafe fn as_mut_ptr(&mut self, min_len: usize) -> *mut u16 {
        // Add one additional codepoint for the NUL terminator
        let min_len = min_len + 1;

        let cur_cap = self.data.capacity();

        if cur_cap < min_len {
            self.data.reserve(min_len - cur_cap);
        }

        self.data.set_len(min_len);

        self.data.as_mut_ptr()
    }

    pub fn to_string(&self) -> String {
        String::from_utf16_lossy(&self.data)
    }
}

#[doc(hidden)]
pub mod consts {
    macro_rules! static_winstr {
        ($(pub static $strname:ident = $strexpr:expr);*;) => (
            lazy_static! {
                $(pub static ref $strname: ::winstr::WinString = ::winstr::WinString::from_str($strexpr);)*
            }
        )
    }

    pub const EMPTY_STR: *const u16 = &0;

    static_winstr! {
        pub static STATIC = "STATIC";
    }
}
