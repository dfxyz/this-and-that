use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

use winapi::um::fileapi::{GetFileAttributesW, SetFileAttributesW, INVALID_FILE_ATTRIBUTES};
use winapi::um::winnt::{FILE_ATTRIBUTE_HIDDEN, FILE_ATTRIBUTE_SYSTEM};

pub fn hide<S: AsRef<OsStr>>(file: S, reverse: bool, system_attr: bool) {
    let file0 = file.as_ref().to_string_lossy();
    let file: Vec<u16> = file
        .as_ref()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let file = file.as_ptr();
    let mut attr = unsafe { GetFileAttributesW(file) };
    if attr == INVALID_FILE_ATTRIBUTES {
        eprintln!("error: failed to get file attributes of '{file0}'");
        return;
    }
    if !reverse {
        attr |= FILE_ATTRIBUTE_HIDDEN;
        if system_attr {
            attr |= FILE_ATTRIBUTE_SYSTEM;
        }
    } else {
        attr &= !FILE_ATTRIBUTE_HIDDEN;
        if system_attr {
            attr &= !FILE_ATTRIBUTE_SYSTEM;
        }
    }
    unsafe {
        if SetFileAttributesW(file, attr) == 0 {
            eprintln!("error: failed to set file attributes of '{file0}'")
        }
    }
}
