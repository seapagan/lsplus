use super::utils::icons::Icon;
use config::Config;
use serde::Deserialize;
use std::convert::From;
use std::path::PathBuf;
use std::time::SystemTime;

macro_rules! config_to_params {
    ($settings:expr, $params:ident, $( $field:ident ),* ) => {
        $(
            if let Ok(value) = $settings.get_bool(stringify!($field)) {
                $params.$field = value;
            }
        )*
    };
}

#[derive(Debug, Deserialize)]
pub struct Params {
    pub show_all: bool,
    pub append_slash: bool,
    pub dirs_first: bool,
    pub almost_all: bool,
    pub long_format: bool,
    pub human_readable: bool,
    pub no_icons: bool,
    pub fuzzy_time: bool,
}

impl Default for Params {
    fn default() -> Self {
        Params {
            show_all: false,
            append_slash: false,
            dirs_first: false,
            almost_all: false,
            long_format: false,
            human_readable: false,
            no_icons: false,
            fuzzy_time: false,
        }
    }
}

impl From<Config> for Params {
    fn from(settings: Config) -> Self {
        let mut params = Params::default();

        config_to_params!(
            settings,
            params,
            show_all,
            append_slash,
            dirs_first,
            almost_all,
            long_format,
            human_readable,
            no_icons,
            fuzzy_time
        );

        params
    }
}

#[derive(Debug)]
pub struct FileInfo {
    pub file_type: String,
    pub mode: String,
    pub nlink: u64,
    pub user: String,
    pub group: String,
    pub size: u64,
    pub mtime: SystemTime,
    pub item_icon: Option<Icon>,
    pub display_name: String,
    pub full_path: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    #[test]
    fn test_params_default() {
        let params = Params::default();
        assert!(!params.show_all);
        assert!(!params.append_slash);
        assert!(!params.dirs_first);
        assert!(!params.almost_all);
        assert!(!params.long_format);
        assert!(!params.human_readable);
        assert!(!params.no_icons);
        assert!(!params.fuzzy_time);
    }

    #[test]
    fn test_params_from_config() {
        let mut settings = Config::default();
        settings
            .set("show_all", true)
            .unwrap()
            .set("append_slash", true)
            .unwrap()
            .set("dirs_first", true)
            .unwrap()
            .set("almost_all", true)
            .unwrap()
            .set("long_format", true)
            .unwrap()
            .set("human_readable", true)
            .unwrap()
            .set("no_icons", true)
            .unwrap()
            .set("fuzzy_time", true)
            .unwrap();

        let params: Params = settings.into();
        assert!(params.show_all);
        assert!(params.append_slash);
        assert!(params.dirs_first);
        assert!(params.almost_all);
        assert!(params.long_format);
        assert!(params.human_readable);
        assert!(params.no_icons);
        assert!(params.fuzzy_time);
    }

    #[test]
    fn test_file_info() {
        let file_info = FileInfo {
            file_type: String::from("-"),
            mode: String::from("rw-r--r--"),
            nlink: 1,
            user: String::from("user"),
            group: String::from("group"),
            size: 1024,
            mtime: SystemTime::now(),
            item_icon: None,
            display_name: String::from("file.txt"),
            full_path: PathBuf::from("/path/to/file.txt"),
        };

        assert_eq!(file_info.file_type, "-");
        assert_eq!(file_info.mode, "rw-r--r--");
        assert_eq!(file_info.nlink, 1);
        assert_eq!(file_info.user, "user");
        assert_eq!(file_info.group, "group");
        assert_eq!(file_info.size, 1024);
        assert!(file_info.item_icon.is_none());
        assert_eq!(file_info.display_name, "file.txt");
        assert_eq!(file_info.full_path, PathBuf::from("/path/to/file.txt"));
    }
}
