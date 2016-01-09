use ::ffi::{BorrowHandle, WindowHandle, WindowEvents};

/// Generic operations supported by all window types.
pub trait AbsWindow: BorrowHandle<W> where W: WindowEvents {
    /// Get the backing pointer to this Window.
    #[doc(hidden)]
    fn ptr(&self) -> WindowPtr; 
}
