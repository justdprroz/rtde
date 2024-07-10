use std::ffi::CString;

use crate::structs::Color;

pub fn argb_to_int(c: Color) -> u64 {
    (c.alpha as u64) << 24 | (c.red as u64) << 16 | (c.green as u64) << 8 | (c.blue as u64)
}

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

macro_rules! log {
    ($($e:expr),+) => {
        #[cfg(debug_assertions)]
        println!($($e),+);
    };
}

pub(crate) use log;

pub fn no_zombies() {
    use nix::sys::signal::*;
    unsafe {
        let sa = SigAction::new(
            SigHandler::SigIgn,
            SaFlags::SA_NOCLDSTOP | SaFlags::SA_NOCLDWAIT | SaFlags::SA_RESTART,
            SigSet::empty(),
        );
        let _ = sigaction(SIGCHLD, &sa);
    }
}
