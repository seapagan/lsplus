use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::sync::OnceLock;
use std::{fmt, fs};

#[allow(dead_code)] // This is a temporary solution to avoid warnings
#[derive(Debug, Clone, Copy)]
pub enum Icon {
    // we define all the possible icons we can use. This will be a growing
    // list as we decode more file types.
    Folder = '\u{f07c}' as isize,
    Symlink = '\u{f1177}' as isize,
    GenericFile = '\u{f15b}' as isize,

    // specific folder types
    SshFolder = '\u{f084}' as isize,
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
    LockFile = '\u{f0221}' as isize,
    LogFile = '\u{f18d}' as isize,
    LuaFile = '\u{e620}' as isize,
    MarkdownFile = '\u{e73e}' as isize,
    PictureFile = '\u{f03e}' as isize,
    PerlFile = '\u{e67e}' as isize,
    PythonFile = '\u{e606}' as isize,
    ReactFile = '\u{e7ba}' as isize,
    RubyFile = '\u{e23e}' as isize,
    RustFile = '\u{e7a8}' as isize,
    SwapFile = '\u{f0fb4}' as isize,
    TerminalFile = '\u{ea85}' as isize,
    TextFile = '\u{f15c}' as isize,
    TomlFile = '\u{e6b2}' as isize,
    TypeScriptFile = '\u{e628}' as isize,
    XmlFile = '\u{e619}' as isize,
    ZipFile = '\u{f1c6}' as isize,
}

impl Icon {
    fn as_char(self) -> char {
        char::from_u32(self as u32).unwrap()
    }

    fn as_string(self) -> String {
        self.as_char().to_string()
    }
}

impl fmt::Display for Icon {
    // implement the Display trait so we can print the icons as strings easily
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

// map folder names to icons
fn folder_icons() -> &'static HashMap<&'static str, Icon> {
    static FOLDER_ICONS: OnceLock<HashMap<&'static str, Icon>> =
        OnceLock::new();

    FOLDER_ICONS.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert(".config", Icon::ConfigFile);
        m.insert(".github", Icon::GitHubFolder);
        m.insert(".ssh", Icon::SshFolder);
        m.insert(".git", Icon::GitFile);
        m.insert(".vscode", Icon::VsCodeFolder);
        m.insert("node_modules", Icon::NodeModulesFolder);
        m.insert("Trash", Icon::TrashFolder);
        m.insert("home", Icon::HomeFolder);

        m
    })
}

// map file extensions to icons
fn file_icons() -> &'static HashMap<&'static str, Icon> {
    static FILE_ICONS: OnceLock<HashMap<&'static str, Icon>> = OnceLock::new();

    FILE_ICONS.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("log", Icon::LogFile);
        m.insert("txt", Icon::TextFile);

        // config files
        m.insert("conf", Icon::ConfigFile);
        m.insert("cfg", Icon::ConfigFile);
        m.insert("ini", Icon::ConfigFile);
        m.insert("gitignore", Icon::GitFile);
        m.insert("gitconfig", Icon::GitFile);

        // formatted text files
        m.insert("json", Icon::JsonFile);
        m.insert("md", Icon::MarkdownFile);
        m.insert("toml", Icon::TomlFile);
        m.insert("xml", Icon::XmlFile);
        m.insert("yaml", Icon::ConfigFile);
        m.insert("yml", Icon::ConfigFile);

        // database files
        m.insert("db", Icon::DatabaseFile);
        m.insert("sqlite", Icon::DatabaseFile);
        m.insert("sql", Icon::DatabaseFile);

        // coding related files
        m.insert("py", Icon::PythonFile);
        m.insert("jsx", Icon::ReactFile);
        m.insert("tsx", Icon::ReactFile);
        m.insert("rb", Icon::RubyFile);
        m.insert("gemrc", Icon::RubyFile);
        m.insert("rs", Icon::RustFile);
        m.insert("ts", Icon::TypeScriptFile);
        m.insert("lua", Icon::LuaFile);
        m.insert("pl", Icon::PerlFile);

        // web-dev related files
        m.insert("css", Icon::CssFile);
        m.insert("html", Icon::HtmlFile);
        m.insert("htm", Icon::HtmlFile);
        m.insert("js", Icon::JavaScriptFile);

        // picture files
        m.insert("jpg", Icon::PictureFile);
        m.insert("png", Icon::PictureFile);
        m.insert("svg", Icon::PictureFile);

        // shell-related files
        m.insert("sh", Icon::TerminalFile);
        m.insert("bash", Icon::TerminalFile);
        m.insert("bashrc", Icon::TerminalFile);
        m.insert("zsh", Icon::TerminalFile);
        m.insert("zshrc", Icon::TerminalFile);
        m.insert("fish", Icon::TerminalFile);
        m.insert("profile", Icon::TerminalFile);
        m.insert("zprofile", Icon::TerminalFile);

        // history files
        m.insert("bash_history", Icon::HistoryFile);
        m.insert("zsh_history", Icon::HistoryFile);
        m.insert("psql_history", Icon::HistoryFile);

        // archive or simiar
        m.insert("deb", Icon::DebianFile);
        m.insert("tar.gz", Icon::ZipFile);
        m.insert("tgz", Icon::ZipFile);

        // lock files
        m.insert("lock", Icon::LockFile);

        m
    })
}

fn known_extensions() -> &'static HashSet<&'static str> {
    // Return a set of all known extensions, from the keys of the file_icons
    // hashmap
    static KNOWN_EXTENSIONS: OnceLock<HashSet<&'static str>> = OnceLock::new();
    KNOWN_EXTENSIONS.get_or_init(|| file_icons().keys().cloned().collect())
}

fn get_folder_icon(folder_name: &str) -> Icon {
    // Use Path to get the folder name
    let path = Path::new(folder_name);
    let folder_name_trimmed = path
        .file_name()
        .unwrap_or(path.as_os_str())
        .to_str()
        .unwrap_or(folder_name);

    // Return the icon for the folder based on its trimmed name
    *folder_icons()
        .get(folder_name_trimmed)
        .unwrap_or(&Icon::Folder)
}

fn get_file_icon(file_name: &str) -> Icon {
    // Find the longest known extension from the end of the filename and return
    // the icon for that extension
    let extension = known_extensions()
        .iter()
        .filter(|&ext| file_name.ends_with(ext))
        .max_by_key(|ext| ext.len())
        .unwrap_or(&"");

    *file_icons().get(*extension).unwrap_or(&Icon::GenericFile)
}

pub fn get_item_icon(metadata: &fs::Metadata, file_name: &str) -> Icon {
    // Return the icon for the item based on its metadata and name
    if metadata.is_dir() {
        // Icon::Folder
        get_folder_icon(file_name)
    } else if metadata.is_symlink() {
        Icon::Symlink
    } else {
        // UnicodeChar::GenericFile
        get_file_icon(file_name)
    }
}
