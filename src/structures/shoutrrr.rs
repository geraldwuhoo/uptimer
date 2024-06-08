use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use log::debug;

use super::errors::UptimersError;

#[link(name = "shoutrrr", kind = "static")]
extern "C" {
    fn Shoutrrr(url: *const c_char, msg: *const c_char) -> *const c_char;
}

pub fn notify(url: &str, msg: String) -> Result<(), UptimersError> {
    debug!("sending shoutrrr notification to {}, msg: {}", url, msg);
    let url = CString::new(url.to_string())?;
    let msg = CString::new(msg)?;

    let result = unsafe { Shoutrrr(url.as_ptr(), msg.as_ptr()) };
    let c_str = unsafe { CStr::from_ptr(result) };
    let string = c_str.to_str()?;
    match string.is_empty() {
        true => Ok(()),
        false => Err(UptimersError::Other(string.to_string())),
    }
}
