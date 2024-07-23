use std::collections::HashMap;
use std::{fmt, fs};

#[allow(dead_code)] // This is a temporary solution to avoid warnings
#[derive(Debug, Clone, Copy)]
pub enum UnicodeChar {
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
    JsonFile = '\u{e60b}' as isize,
    LogFile = '\u{f18d}' as isize,
    LuaFile = '\u{e620}' as isize,
    MarkdownFile = '\u{e73e}' as isize,
    PictureFile = '\u{f03e}' as isize,
    PythonFile = '\u{e606}' as isize,
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
        char::from_u32(self as u32).unwrap()
    }

    fn as_string(self) -> String {
        self.as_char().to_string()
    }
}

impl fmt::Display for UnicodeChar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

// Define a function that maps file names to icons
fn get_file_icon(file_name: &str) -> UnicodeChar {
    let mut map = HashMap::new();
    map.insert("conf", UnicodeChar::ConfigFile);
    map.insert("log", UnicodeChar::LogFile);

    //formatted text files
    map.insert("json", UnicodeChar::JsonFile);
    map.insert("md", UnicodeChar::MarkdownFile);
    map.insert("toml", UnicodeChar::TomlFile);
    map.insert("xml", UnicodeChar::XmlFile);

    // coding related files
    map.insert("py", UnicodeChar::PythonFile);
    map.insert("jsx", UnicodeChar::ReactFile);
    map.insert("tsx", UnicodeChar::ReactFile);
    map.insert("rb", UnicodeChar::RubyFile);
    map.insert("rs", UnicodeChar::RustFile);
    map.insert("ts", UnicodeChar::TypeScriptFile);
    map.insert("lua", UnicodeChar::LuaFile);

    // we-dev related files
    map.insert("css", UnicodeChar::CssFile);
    map.insert("html", UnicodeChar::HtmlFile);
    map.insert("htm", UnicodeChar::HtmlFile);
    map.insert("js", UnicodeChar::JavaScriptFile);

    // picture files
    map.insert("jpg", UnicodeChar::PictureFile);
    map.insert("png", UnicodeChar::PictureFile);
    map.insert("svg", UnicodeChar::PictureFile);

    // shell-related files
    map.insert("sh", UnicodeChar::TerminalFile);
    map.insert("bash", UnicodeChar::TerminalFile);
    map.insert("zsh", UnicodeChar::TerminalFile);
    map.insert("fish", UnicodeChar::TerminalFile);
    map.insert("profile", UnicodeChar::TerminalFile);

    let extension = file_name.split('.').last().unwrap_or("");
    // Return the icon or default to GenericFile if not found
    *map.get(extension).unwrap_or(&UnicodeChar::GenericFile)
}
pub fn get_item_icon(metadata: &fs::Metadata, file_name: &str) -> UnicodeChar {
    if metadata.is_dir() {
        UnicodeChar::Folder
    } else if metadata.is_symlink() {
        UnicodeChar::Symlink
    } else {
        // UnicodeChar::GenericFile
        get_file_icon(file_name)
    }
}
