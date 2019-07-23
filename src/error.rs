use std::{fmt, mem::MaybeUninit, ptr};

use winapi::um::{
    errhandlingapi,
    winbase::{FormatMessageW, FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_IGNORE_INSERTS},
    winnt::LANG_SYSTEM_DEFAULT,
};

use crate::utils;

pub(crate) struct WindowsApiError {
    code: u32,
}

pub(crate) type AppResult<T> = std::result::Result<T, WindowsApiError>;

impl WindowsApiError {
    pub fn last() -> Self {
        WindowsApiError {
            code: unsafe { errhandlingapi::GetLastError() },
        }
    }
}

impl fmt::Display for WindowsApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut err_msg = MaybeUninit::<[u16; 512]>::uninit();
        let err_msg_len = unsafe {
            FormatMessageW(
                FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
                ptr::null_mut(),
                self.code,
                LANG_SYSTEM_DEFAULT as _,
                err_msg.as_mut_ptr() as _,
                256,
                ptr::null_mut(),
            )
        };

        if err_msg_len == 0 {
            write!(f, "{}", "unknown error")
        } else {
            let err_msg = unsafe { err_msg.assume_init() };
            write!(
                f,
                "{}",
                utils::string_from_wide(&err_msg[0..err_msg_len as _])
            )
        }
    }
}

impl fmt::Debug for WindowsApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[macro_export]
macro_rules! last_error {
    () => {
        Err(WindowsApiError::last())
    };
}
