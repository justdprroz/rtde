//! Some utility functions without much logic in them

use std::ffi::CString;

use crate::structs::Color;

/// Convert color to 64 bit int for x11
pub fn argb_to_int(c: Color) -> u64 {
    (c.alpha as u64) << 24 | (c.red as u64) << 16 | (c.green as u64) << 8 | (c.blue as u64)
}

/// Convert Rust Vector of Strings to C array of bytes
pub fn vec_string_to_bytes(strings: Vec<String>) -> Vec<u8> {
    let mut bytes: Vec<u8> = vec![];
    for string in strings {
        match CString::new(string) {
            Ok(c) => bytes.append(&mut c.into_bytes_with_nul()),
            Err(_) => todo!(),
        }
    }
    bytes
}

/// Log if in debug
#[macro_export]
macro_rules! log {
    ($($e:expr),+) => {
        #[cfg(debug_assertions)]
        println!($($e),+);
    };
}
pub use log;
