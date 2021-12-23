use std::{fmt, error};
use windows::core::Error;

/// The error type for when the OS cannot perform the requested operation.
#[derive(Debug)]
pub struct Win32Error {
    line: u32,
    file: &'static str,
    error: Error,
}

impl Win32Error {
    #[allow(dead_code)]
    pub(crate) fn new(line: u32, file: &'static str, error: Error) -> Win32Error {
        Win32Error { line, file, error }
    }
}

#[allow(unused_macros)]
macro_rules! win_error {
    ($error:expr) => {{
            crate::error::Win32Error::new(line!(), file!(), $error)
        }};
}

impl fmt::Display for Win32Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.pad(&format!(
            "os error at {}:{}: {}",
            self.file, self.line, self.error
        ))
    }
}

impl error::Error for Win32Error {}
