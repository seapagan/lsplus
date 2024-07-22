use std::fs;

use std::fmt;

#[derive(Debug, Clone, Copy)]
enum UnicodeChar {
    Folder = '\u{f07c}' as isize,
    Symlink = '\u{f1177}' as isize,

    GenericFile = '\u{f15b}' as isize,
}

impl UnicodeChar {
    fn as_char(self) -> char {
        // This is safe because we know all our values are valid Unicode code points
        char::from_u32(self as u32).unwrap()
    }
}

impl fmt::Display for UnicodeChar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

pub fn get_item_icon(metadata: &fs::Metadata) -> String {
    if metadata.is_dir() {
        UnicodeChar::Folder.to_string()
    } else if metadata.is_symlink() {
        UnicodeChar::Symlink.to_string()
    } else {
        UnicodeChar::GenericFile.to_string()
    }
}
