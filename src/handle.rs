use std::convert;

use winapi::um::{handleapi::CloseHandle, winnt::HANDLE};

use crate::error::{AppResult, WindowsApiError};

#[repr(transparent)]
pub(crate) struct Handle {
    h: HANDLE,
}

impl Handle {
    pub fn new(h: HANDLE) -> AppResult<Self> {
        if h.is_null() {
            Err(WindowsApiError::last())
        } else {
            // be careful for INVALID_HANDLE_VALUE which may be valid too
            // see: https://devblogs.microsoft.com/oldnewthing/20040302-00/?p=40443
            Ok(Handle { h })
        }
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.h) };
    }
}

impl convert::From<&Handle> for HANDLE {
    fn from(hdl: &Handle) -> Self {
        hdl.h
    }
}
