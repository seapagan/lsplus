use std::fs;

use std::fmt;

#[allow(dead_code)] // This is a temporary solution to avoid warnings
#[derive(Debug, Clone, Copy)]
enum UnicodeChar {
    // we define all the possible icons we can use. This will be a growing
    // list as we decode more file types.
    Folder = '\u{f07c}' as isize,
    Symlink = '\u{f1177}' as isize,
    GenericFile = '\u{f15b}' as isize,

    // specific folder types
    ConfigFolder = '\u{e5fc}' as isize,
    GitFolder = '\u{f1d3}' as isize,
    GitHubFolder = '\u{f408}' as isize,
    HomeFolder = '\u{f015}' as isize,
    NodeModulesFolder = '\u{ed0d}' as isize,
    TrashFolder = '\u{ea81}' as isize,
    VsCodeFolder = '\u{f0a1e}' as isize,

    // specific file types
    ConfigFile = '\u{f013}' as isize,
    CssFile = '\u{e749}' as isize,
    HtmlFile = '\u{e736}' as isize,
    JavaScriptFile = '\u{e74e}' as isize,
    JsonFile = '\u{e626}' as isize,
    LogFile = '\u{f4ed}' as isize,
    MarkdownFile = '\u{e73e}' as isize,
    PictureFile = '\u{f03e}' as isize,
    PythonFile = '\u{ed1b}' as isize,
    ReactFile = '\u{e7ba}' as isize,
    RubyFile = '\u{e23e}' as isize,
    RustFile = '\u{e7a8}' as isize,
    TerminalFile = '\u{ea85}' as isize,
    TomlFile = '\u{e6b2}' as isize,
    TypeScriptFile = '\u{e628}' as isize,
    XmlFile = '\u{e619}' as isize,
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
