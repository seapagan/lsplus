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
