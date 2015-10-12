use user32;

use abs_window::AbsWindow;
use winstr::WinString;

pub mod label;
pub mod edit;

pub use self::label::Label;

pub unsafe trait TextWindow: AbsWindow {
    fn get_text(&self) -> String {
        let mut win_str = WinString::empty();

        unsafe {
            let min_len = user32::GetWindowTextLengthW(self.ptr());
            let winstr_ptr = win_str.as_mut_ptr(min_len as usize);

            // Add 1 for NUL terminator
            user32::GetWindowTextW(self.ptr(), winstr_ptr, min_len + 1);
        }

        win_str.to_string()
    }

    fn set_text(&mut self, text: &str) {
        let win_str = WinString::from_str(text);

        unsafe {
            user32::SetWindowTextW(self.ptr(), win_str.as_ptr());
        }
    }
}
