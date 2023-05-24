use std::ffi::{OsString, OsStr};
#[cfg(windows)]
use std::os::windows::prelude::OsStrExt;
#[cfg(not(windows))]
use std::os::unix::prelude::OsStrExt;

use glob_match::glob_match;
use ignore::DirEntry;
#[cfg(windows)]
use byte_slice_cast::AsByteSlice;

pub fn filter_c_files(entry: &DirEntry) -> bool {
    let path = entry.path();
    if path.is_dir() {
        return true;
    }
    path.to_str()
        .map_or(false, |s| glob_match("*/**/*.{c,cpp,h,hpp,ino}", s))
}


#[cfg(windows)]
pub fn join_filepath_list(filepaths: Vec<OsString>) -> Vec<u8> {
    let filepath_list_windows = filepaths.join(OsStr::new("\n")).encode_wide().collect::<Vec<u16>>();
    filepath_list_windows.as_slice().as_byte_slice().to_vec()
}

#[cfg(not(windows))]
pub fn join_filepath_list(filepaths: Vec<OsString>) -> Vec<u8> {
    filepaths.join(OsStr::new("\n")).as_bytes().to_vec()
}
