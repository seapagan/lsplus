use std::path::PathBuf;

use config::{Config, File, FileFormat};
use dirs_next::home_dir;

use crate::Params;

fn config_path() -> Option<PathBuf> {
    config_path_from_home(home_dir())
}

#[doc(hidden)]
pub fn config_path_from_home(home: Option<PathBuf>) -> Option<PathBuf> {
    let mut path = home?;
    path.push(".config/lsplus/config.toml");
    Some(path)
}

pub fn load_config() -> Params {
    load_config_from_path(config_path())
}

#[doc(hidden)]
pub fn load_config_from_path(config_path: Option<PathBuf>) -> Params {
    let Some(config_path) = config_path else {
        return Params::default();
    };

    if !config_path.is_file() {
        return Params::default();
    }

    let settings = Config::builder()
        .add_source(File::from(config_path).format(FileFormat::Toml))
        .build();

    match settings {
        Ok(config) => config.into(),
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            Params::default()
        }
    }
}
