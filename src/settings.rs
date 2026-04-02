use std::path::PathBuf;

use config::{Config, File, FileFormat};
use dirs_next::home_dir;

use crate::Params;

fn config_path() -> Option<PathBuf> {
    let mut path = home_dir()?;
    path.push(".config/lsplus/config.toml");
    Some(path)
}

pub fn load_config() -> Params {
    load_config_from_path(config_path())
}

fn load_config_from_path(config_path: Option<PathBuf>) -> Params {
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_load_config_default() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("missing-config.toml");

        let config = load_config_from_path(Some(config_path));

        assert_eq!(config, Params::default());
    }

    #[test]
    fn test_load_config_error() {
        let config = load_config_from_path(None);

        assert_eq!(config, Params::default());
    }

    #[test]
    fn test_load_config_error_other() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join(".config/lsplus/config.toml");
        std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        std::fs::write(&config_path, "invalid = toml [ content").unwrap();

        let config = load_config_from_path(Some(config_path));

        assert_eq!(config, Params::default());
    }
}
