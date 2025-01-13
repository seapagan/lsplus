use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::sync::OnceLock;
use std::{fmt, fs};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Icon {
    // we define all the possible icons we can use. This will be a growing
    // list as we decode more file types.
    Folder = '\u{f07c}' as isize,
    Symlink = '\u{f1177}' as isize,
    GenericFile = '\u{f15b}' as isize,

    // specific folder types
    CacheFolder = '\u{f163f}' as isize,
    GitHubFolder = '\u{f408}' as isize,
    HomeFolder = '\u{f015}' as isize,
    NodeModulesFolder = '\u{f0399}' as isize,
    SecurityFolder = '\u{f084}' as isize,
    TrashFolder = '\u{ea81}' as isize,
    VsCodeFolder = '\u{f0a1e}' as isize,

    // specific file types
    CompactDiscFile = '\u{e271}' as isize,
    ConfigFile = '\u{f013}' as isize,
    CssFile = '\u{e749}' as isize,
    DatabaseFile = '\u{e706}' as isize,
    DebianFile = '\u{f306}' as isize,
    DockerFile = '\u{f21f}' as isize,
    FontFile = '\u{e659}' as isize,
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
    SassFile = '\u{e603}' as isize,
    SwapFile = '\u{f0fb4}' as isize,
    TerminalFile = '\u{ea85}' as isize,
    TextFile = '\u{f15c}' as isize,
    TomlFile = '\u{e6b2}' as isize,
    TypeScriptFile = '\u{e628}' as isize,
    WrenchFile = '\u{f0ad}' as isize,
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
        m.insert(".ssh", Icon::SecurityFolder);
        m.insert(".git", Icon::GitFile);
        m.insert(".vscode", Icon::VsCodeFolder);
        m.insert("node_modules", Icon::NodeModulesFolder);
        m.insert("Trash", Icon::TrashFolder);
        m.insert("home", Icon::HomeFolder);
        m.insert("root", Icon::SecurityFolder);
        m.insert("venv", Icon::PythonFile);
        m.insert(".venv", Icon::PythonFile);
        m.insert(".pyenv", Icon::PythonFile);
        m.insert(".rbenv", Icon::RubyFile);
        m.insert(".npm", Icon::NodeModulesFolder);
        m.insert(".yarn", Icon::NodeModulesFolder);
        m.insert(".cargo", Icon::RustFile);
        m.insert(".rustup", Icon::RustFile);
        m.insert(".gnupg", Icon::SecurityFolder);
        m.insert(".docker", Icon::DockerFile);
        m.insert(".cpan", Icon::PerlFile);
        m.insert(".cpanm", Icon::PerlFile);
        m.insert(".cache", Icon::CacheFolder);

        m
    })
}

// map file NAME extensions to icons
fn file_name_icons() -> &'static HashMap<&'static str, Icon> {
    // these are specifc exact file names that we want to map to icons.
    static FILE_NAME_ICONS: OnceLock<HashMap<&'static str, Icon>> =
        OnceLock::new();

    FILE_NAME_ICONS.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("swapfile", Icon::SwapFile);
        m.insert("docker-compose.yml", Icon::DockerFile);
        m.insert("Dockerfile", Icon::DockerFile);
        m.insert("LICENSE", Icon::TextFile);
        m.insert("Rakefile", Icon::RubyFile);
        m.insert("Gemfile", Icon::RubyFile);

        m
    })
}

// map file EXTENSIONS to icons
fn file_type_icons() -> &'static HashMap<&'static str, Icon> {
    static FILE_ICONS: OnceLock<HashMap<&'static str, Icon>> = OnceLock::new();

    // this one is done a bit differently since there may be many extensions
    // sharing the same icon
    FILE_ICONS.get_or_init(|| {
        let icon_groups: Vec<(&[&str], Icon)> = vec![
            (&["txt"], Icon::LogFile),
            (&["log"], Icon::TextFile),
            (
                &["conf", "cfg", "ini", "pylintrc", "yaml", "yml", "yarnrc"],
                Icon::ConfigFile,
            ),
            (
                &["gitignore", "gitconfig", "gitattributes", "gitmodules"],
                Icon::GitFile,
            ),
            (&["env"], Icon::WrenchFile),
            (&["json"], Icon::JsonFile),
            (&["md"], Icon::MarkdownFile),
            (&["toml"], Icon::TomlFile),
            (&["xml"], Icon::XmlFile),
            (&["db", "sqlite", "sql"], Icon::DatabaseFile),
            (&["py", "whl"], Icon::PythonFile),
            (&["jsx", "tsx"], Icon::ReactFile),
            (&["rb", "gemrc", "rspec"], Icon::RubyFile),
            (&["rs"], Icon::RustFile),
            (&["ts"], Icon::TypeScriptFile),
            (&["lua"], Icon::LuaFile),
            (&["pl"], Icon::PerlFile),
            (&["css"], Icon::CssFile),
            (&["scss", "sass"], Icon::SassFile),
            (&["html", "htm"], Icon::HtmlFile),
            (&["js", "cjs"], Icon::JavaScriptFile),
            (&["jpg", "png", "svg"], Icon::PictureFile),
            (
                &[
                    "sh", "bash", "bashrc", "zsh", "zshrc", "fish", "profile",
                    "zprofile",
                ],
                Icon::TerminalFile,
            ),
            (
                &["bash_history", "zsh_history", "psql_history"],
                Icon::HistoryFile,
            ),
            (&["deb"], Icon::DebianFile),
            (
                &[
                    "gz", "tgz", "zip", "rar", "xz", "tar", "7z", "bz2",
                    "bz2", "z", "Z", "arj", "lzh", "cab",
                ],
                Icon::ZipFile,
            ),
            (
                &["iso", "bin", "dmg", "img", "qcow", "vdi", "vmdk"],
                Icon::CompactDiscFile,
            ),
            (&["lock"], Icon::LockFile),
            (
                &[
                    "ttf", "otf", "woff", "woff2", "eot", "pfb", "pfm", "fon",
                    "dfont", "pfa", "pcf", "bdf", "snf",
                ],
                Icon::FontFile,
            ),
        ];

        let mut m = HashMap::new();
        for (extensions, icon) in icon_groups {
            for &ext in extensions {
                m.insert(ext, icon);
            }
        }
        m
    })
}

fn known_extensions() -> &'static HashSet<&'static str> {
    // Return a set of all known extensions, from the keys of the file_icons
    // hashmap
    static KNOWN_EXTENSIONS: OnceLock<HashSet<&'static str>> = OnceLock::new();
    KNOWN_EXTENSIONS
        .get_or_init(|| file_type_icons().keys().cloned().collect())
}

fn get_folder_icon(folder_name: &str) -> Icon {
    *folder_icons().get(folder_name).unwrap_or(&Icon::Folder)
}

// Helper function to check if a file name ends with an extension
fn has_extension(file_name: &str, ext: &str) -> bool {
    // Guard against empty extension
    if ext.is_empty() {
        return false;
    }

    // Guard against special dot cases
    if file_name == "." || file_name == ".." {
        return false;
    }

    file_name.ends_with(ext)
        && file_name.len() > ext.len()
        && file_name[..file_name.len() - ext.len()].ends_with('.')
        && file_name.trim_start_matches('.').len() > ext.len() + 1
}

fn get_file_icon(file_name: &str) -> Icon {
    // Find the longest known extension from the end of the filename and return
    // the icon for that extension

    let extension = known_extensions()
        .iter()
        .filter(|&&ext| has_extension(file_name, ext))
        .max_by_key(|ext| ext.len())
        .unwrap_or(&"");

    *file_type_icons()
        .get(*extension)
        .unwrap_or(&Icon::GenericFile)
}

fn get_filename_icon(file_name: &str) -> Option<Icon> {
    // Return the icon for the filename based on its name if found
    file_name_icons().get(file_name).cloned()
}

pub fn get_item_icon(metadata: &fs::Metadata, file_path: &str) -> Icon {
    // Extract just the file name without the path
    let file_name = Path::new(file_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    // Return the icon for the item based on its metadata and name
    if metadata.is_dir() {
        // Icon::Folder
        get_folder_icon(file_name)
    } else if metadata.is_symlink() {
        Icon::Symlink
    } else {
        get_filename_icon(file_name)
            .unwrap_or_else(|| get_file_icon(file_name))
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_extension() {
        // Standard cases - should match
        assert!(has_extension("test.txt", "txt"));
        assert!(has_extension("path/to/file.rs", "rs"));
        assert!(has_extension(".hidden.conf", "conf"));
        assert!(has_extension("multiple.dots.md", "md"));

        // Standard cases - should not match
        assert!(!has_extension("text", "txt"));
        assert!(!has_extension("notanexe", "exe"));
        assert!(!has_extension(".gitignore", "git"));
        assert!(!has_extension("Makefile", "file"));

        // Edge cases - should not match
        assert!(!has_extension("", "txt")); // empty string
        assert!(!has_extension(".", "")); // just a dot
        assert!(!has_extension("..", ".")); // double dot
        assert!(!has_extension("txt", "txt")); // filename equals extension
        assert!(!has_extension(".txt", "txt")); // hidden file equals extension

        // Extension-like cases - should not match
        assert!(!has_extension("txt.bak", "txt")); // extension in middle
        assert!(!has_extension("mytxt", "txt")); // suffix but no dot
        assert!(!has_extension(".txt.swp", "txt")); // hidden with different ext

        // Special characters
        assert!(has_extension("test.t.x.t", "t"));
        assert!(has_extension("$^#@.bin", "bin"));
        assert!(has_extension("spaces in name.doc", "doc"));
    }

    #[test]
    fn test_get_item_icon() {
        use std::fs::Metadata;
        use std::os::unix::fs::MetadataExt;
        use std::os::unix::fs::FileTypeExt;

        // Create a mock metadata for a file
        let metadata = fs::metadata("Cargo.toml").unwrap();
        
        // Test unknown file type returns generic icon
        let icon = get_item_icon(&metadata, "test.unknown");
        assert_eq!(icon, Icon::GenericFile);

        // Test with a known file type
        let icon = get_item_icon(&metadata, "test.rs");
        assert_eq!(icon, Icon::RustFile);

        // Test with a known filename
        let icon = get_item_icon(&metadata, "Cargo.toml");
        assert_eq!(icon, Icon::TomlFile);

        // Test with a directory
        let metadata = fs::metadata(".").unwrap();
        let icon = get_item_icon(&metadata, "test_dir");
        assert_eq!(icon, Icon::Folder);

        // Test with a symlink
        // Create a temporary symlink for testing
        let _ = std::os::unix::fs::symlink("test_target", "test_link");
        if let Ok(metadata) = fs::symlink_metadata("test_link") {
            let icon = get_item_icon(&metadata, "test_link");
            assert_eq!(icon, Icon::Symlink);
        }
        // Clean up the symlink
        let _ = fs::remove_file("test_link");
    }

    #[test]
    fn test_get_filename_icon() {
        // Test known filenames from file_name_icons
        let icons = file_name_icons();
        assert_eq!(icons.get("swapfile"), Some(&Icon::SwapFile));
        assert_eq!(icons.get("docker-compose.yml"), Some(&Icon::DockerFile));
        assert_eq!(icons.get("Dockerfile"), Some(&Icon::DockerFile));
        assert_eq!(icons.get("LICENSE"), Some(&Icon::TextFile));
        assert_eq!(icons.get("Rakefile"), Some(&Icon::RubyFile));
        assert_eq!(icons.get("Gemfile"), Some(&Icon::RubyFile));
        
        // Test unknown filename
        assert_eq!(icons.get("unknown.txt"), None);
    }

    #[test]
    fn test_folder_icons() {
        // Test known folder names
        assert_eq!(folder_icons().get(".git"), Some(&Icon::GitFile));
        assert_eq!(folder_icons().get("node_modules"), Some(&Icon::NodeModulesFolder));
        assert_eq!(folder_icons().get(".vscode"), Some(&Icon::VsCodeFolder));
        
        // Test unknown folder name
        assert_eq!(folder_icons().get("unknown_folder"), None);
    }
}
