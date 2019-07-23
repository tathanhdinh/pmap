use std::{convert, fmt, io, mem::MaybeUninit, ptr, error};

use winapi::um::{
    errhandlingapi,
    winbase::{FormatMessageW, FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_IGNORE_INSERTS},
    winnt::LANG_SYSTEM_DEFAULT,
};

use crate::utils;

pub(crate) enum AppError {
    WindowsApi { code: u32 },
    Io(io::Error),
}

impl AppError {
    pub fn last_api_error() -> Self {
        AppError::WindowsApi {
            code: unsafe { errhandlingapi::GetLastError() },
        }
    }
}

impl convert::From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        AppError::Io(err)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use AppError::*;
        match self {
            WindowsApi { code } => {
                let mut err_msg = MaybeUninit::<[u16; 512]>::uninit();
                let err_msg_len = unsafe {
                    FormatMessageW(
                        FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
                        ptr::null_mut(),
                        *code,
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

            Io(err) => write!(f, "{}", err),
        }
    }
}

impl fmt::Debug for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl error::Error for AppError {}

pub(crate) struct WindowsApiError {
    code: Option<u32>,
}

pub(crate) type AppResult<T> = std::result::Result<T, WindowsApiError>;

impl WindowsApiError {
    pub fn last() -> Self {
        WindowsApiError {
            code: Some(unsafe { errhandlingapi::GetLastError() }),
        }
    }
}

impl fmt::Display for WindowsApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(code) = self.code {
            let mut err_msg = MaybeUninit::<[u16; 512]>::uninit();
            let err_msg_len = unsafe {
                FormatMessageW(
                    FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
                    ptr::null_mut(),
                    code,
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
        } else {
            write!(f, "{}", "other error")
        }
    }
}

impl fmt::Debug for WindowsApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl convert::From<io::Error> for WindowsApiError {
    fn from(err: io::Error) -> Self {
        WindowsApiError { code: None }
    }
}

#[macro_export]
macro_rules! last_error {
    () => {
        Err(WindowsApiError::last())
    };
}
