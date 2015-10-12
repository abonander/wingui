use ::WindowPtr;

/// Generic operations supported by all widget types.
/// 
pub unsafe trait AbsWindow {
    /// Get the backing pointer to this Window.
    #[doc(hidden)]
    fn ptr(&self) -> WindowPtr; 
}
