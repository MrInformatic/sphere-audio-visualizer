use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

/// Wraps a [`RawWindowHandle`] to implement [`HasRawWindowHandle`]
pub struct RawWindowHandleWrapper(RawWindowHandle);

impl<'a, RWH: HasRawWindowHandle> From<&'a RWH> for RawWindowHandleWrapper {
    fn from(has_raw_window_handle: &'a RWH) -> Self {
        Self(has_raw_window_handle.raw_window_handle())
    }
}

impl From<RawWindowHandle> for RawWindowHandleWrapper {
    fn from(raw_window_handle: RawWindowHandle) -> Self {
        Self(raw_window_handle)
    }
}

unsafe impl HasRawWindowHandle for RawWindowHandleWrapper {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.0.clone()
    }
}
