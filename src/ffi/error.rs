use winapi::*;
use kernel32;

use std::error::Error;
use std::{fmt, ptr, slice};

#[derive(Clone, Debug)]
pub enum WindowsError {
    Code(DWORD),
    Msg(String),
}

impl WindowsError {
    pub fn last() -> Self {
        let err_code = last_error_code(); 
        Self::from_res(get_error_msg(err_code))        
    }

    fn from_res(res: Result<String, DWORD>) -> Self {
        res.map(WindowsError::Msg).unwrap_or_else(WindowsError::Code)
    }
}

impl fmt::Display for WindowsError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use self::WindowsError::*;
        match *self {
            Code(code) => fmt.write_fmt(format_args!("Windows error code: {:X}", code)),
            Msg(ref msg) => fmt.write_fmt(format_args!("Windows error: \"{}\"", msg)),
        }            
    }
}

impl Error for WindowsError {
    fn description(&self) -> &str {
        use self::WindowsError::*;

        match *self {
            Code(code) => "Windows error code",
            Msg(ref msg) => msg,
        }
    }
}

fn last_error_code() -> DWORD {
    unsafe {
        kernel32::GetLastError()
    }
}

fn get_error_msg(err_code: DWORD) -> Result<String, DWORD> {
    const FMT_FLAGS: DWORD = FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM;

    let mut buf_ptr: *const u16 = ptr::null_mut();

    let buf = unsafe {
        let len = kernel32::FormatMessageW(
            FMT_FLAGS, ptr::null(), err_code, 0,
            &mut buf_ptr as *const _ as LPWSTR,
            0, ptr::null_mut()
        ) as usize;

        if len == 0 {
            return Err(last_error_code());
        }

        slice::from_raw_parts(buf_ptr, len)
    };

    let ret = String::from_utf16_lossy(buf);
    
    unsafe {
        if kernel32::HeapFree(kernel32::GetProcessHeap(), 0, buf_ptr as *mut _) == 0 {
            return Err(last_error_code());
        }
    }

    Ok(ret)
}
