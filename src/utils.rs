use std::{ffi::OsString, os::windows::ffi::OsStringExt};

pub fn string_from_wide(s: &[u16]) -> String {
    OsString::from_wide(s).into_string().unwrap()
}
