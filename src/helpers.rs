use glob_match::glob_match;
use ignore::DirEntry;

pub fn filter_c_files(entry: &DirEntry) -> bool {
    let path = entry.path();
    if path.is_dir() {
        return true;
    }
    path.to_str()
        .map_or(false, |s| glob_match("*/**/*.{c,cpp,h,hpp,ino}", s))
}
