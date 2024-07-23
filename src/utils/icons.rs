use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::OnceLock;
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
    GitHubFolder = '\u{f408}' as isize,
    HomeFolder = '\u{f015}' as isize,
    NodeModulesFolder = '\u{ed0d}' as isize,
    TrashFolder = '\u{ea81}' as isize,
    VsCodeFolder = '\u{f0a1e}' as isize,

    // specific file types
    ConfigFile = '\u{f013}' as isize,
    CssFile = '\u{e749}' as isize,
    DatabaseFile = '\u{e706}' as isize,
    DebianFile = '\u{f306}' as isize,
    GitFile = '\u{f1d3}' as isize,
    HistoryFile = '\u{f1da}' as isize,
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
    TextFile = '\u{f15c}' as isize,
    TomlFile = '\u{e6b2}' as isize,
    TypeScriptFile = '\u{e628}' as isize,
    XmlFile = '\u{e619}' as isize,
    ZipFile = '\u{f1c6}' as isize,
}
#[allow(non_upper_case_globals)] // needed to keep constant names consistent
#[allow(dead_code)] // This is a temporary solution to avoid warnings
impl UnicodeChar {
    // these constants are used to make it easier to reference the icons
    // by multiple names without Enum errors. We need this to use the same icon
    // for a folder and a file, for example.
    pub const GitFolder: UnicodeChar = UnicodeChar::GitFile;

    fn as_char(self) -> char {
        char::from_u32(self as u32).unwrap()
    }

    fn as_string(self) -> String {
        self.as_char().to_string()
    }
}

impl fmt::Display for UnicodeChar {
    // implement the Display trait so we can print the icons as strings easily
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

// map extensions to icons
fn file_icons() -> &'static HashMap<&'static str, UnicodeChar> {
    static FILE_ICONS: OnceLock<HashMap<&'static str, UnicodeChar>> =
        OnceLock::new();

    FILE_ICONS.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("log", UnicodeChar::LogFile);
        m.insert("txt", UnicodeChar::TextFile);

        // config files
        m.insert("conf", UnicodeChar::ConfigFile);
        m.insert("cfg", UnicodeChar::ConfigFile);
        m.insert("ini", UnicodeChar::ConfigFile);
        m.insert("gitignore", UnicodeChar::GitFile);
        m.insert("gitconfig", UnicodeChar::GitFile);

        // formatted text files
        m.insert("json", UnicodeChar::JsonFile);
        m.insert("md", UnicodeChar::MarkdownFile);
        m.insert("toml", UnicodeChar::TomlFile);
        m.insert("xml", UnicodeChar::XmlFile);
        m.insert("yaml", UnicodeChar::ConfigFile);
        m.insert("yml", UnicodeChar::ConfigFile);

        // database files
        m.insert("db", UnicodeChar::DatabaseFile);
        m.insert("sqlite", UnicodeChar::DatabaseFile);
        m.insert("sql", UnicodeChar::DatabaseFile);

        // coding related files
        m.insert("py", UnicodeChar::PythonFile);
        m.insert("jsx", UnicodeChar::ReactFile);
        m.insert("tsx", UnicodeChar::ReactFile);
        m.insert("rb", UnicodeChar::RubyFile);
        m.insert("gemrc", UnicodeChar::RubyFile);
        m.insert("rs", UnicodeChar::RustFile);
        m.insert("ts", UnicodeChar::TypeScriptFile);
        m.insert("lua", UnicodeChar::LuaFile);

        // web-dev related files
        m.insert("css", UnicodeChar::CssFile);
        m.insert("html", UnicodeChar::HtmlFile);
        m.insert("htm", UnicodeChar::HtmlFile);
        m.insert("js", UnicodeChar::JavaScriptFile);

        // picture files
        m.insert("jpg", UnicodeChar::PictureFile);
        m.insert("png", UnicodeChar::PictureFile);
        m.insert("svg", UnicodeChar::PictureFile);

        // shell-related files
        m.insert("sh", UnicodeChar::TerminalFile);
        m.insert("bash", UnicodeChar::TerminalFile);
        m.insert("bashrc", UnicodeChar::TerminalFile);
        m.insert("zsh", UnicodeChar::TerminalFile);
        m.insert("zshrc", UnicodeChar::TerminalFile);
        m.insert("fish", UnicodeChar::TerminalFile);
        m.insert("profile", UnicodeChar::TerminalFile);
        m.insert("zprofile", UnicodeChar::TerminalFile);

        // history files
        m.insert("bash_history", UnicodeChar::HistoryFile);
        m.insert("zsh_history", UnicodeChar::HistoryFile);
        m.insert("psql_history", UnicodeChar::HistoryFile);

        // archive or simiar
        m.insert("deb", UnicodeChar::DebianFile);
        m.insert("tar.gz", UnicodeChar::ZipFile);

        m
    })
}

fn get_file_icon(file_name: &str) -> UnicodeChar {
    // Get the known extensions from the file_icons HashMap
    let known_extensions: HashSet<&str> =
        file_icons().keys().cloned().collect();

    // Find the longest known extension from the end of the filename
    let extension = known_extensions
        .iter()
        .filter(|&ext| file_name.ends_with(ext))
        .max_by_key(|ext| ext.len())
        .unwrap_or(&"");

    *file_icons()
        .get(*extension)
        .unwrap_or(&UnicodeChar::GenericFile)
}

// fn get_file_icon(file_name: &str) -> UnicodeChar {
//     let extension = file_name.split('.').last().unwrap_or("");
//     *file_icons()
//         .get(extension)
//         .unwrap_or(&UnicodeChar::GenericFile)
// }

// fn get_file_icon(file_name: &str) -> UnicodeChar {
//     let extension =
//         file_name.split_once('.').map(|(_, ext)| ext).unwrap_or("");
//     *file_icons()
//         .get(extension)
//         .unwrap_or(&UnicodeChar::GenericFile)
// }

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
