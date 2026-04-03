use super::utils::icons::Icon;
use config::Config;
use serde::Deserialize;
use std::convert::From;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::cli;

macro_rules! config_to_params {
    ($settings:expr_2021, $params:ident, $( $field:ident ),* ) => {
        $(
            if let Ok(value) = $settings.get_bool(stringify!($field)) {
                $params.$field = value;
            }
        )*
    };
}

#[derive(Debug, Deserialize, PartialEq, Default)]
pub struct Params {
    pub show_all: bool,
    pub append_slash: bool,
    pub dirs_first: bool,
    pub almost_all: bool,
    pub long_format: bool,
    pub human_readable: bool,
    pub no_icons: bool,
    pub gitignore: bool,
    pub fuzzy_time: bool,
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
            gitignore,
            fuzzy_time
        );

        params
    }
}

impl Params {
    pub fn merge(flags: &cli::Flags, config: &Self) -> Self {
        Self {
            show_all: flags.show_all || config.show_all,
            append_slash: flags.slash || config.append_slash,
            dirs_first: flags.dirs_first || config.dirs_first,
            almost_all: flags.almost_all || config.almost_all,
            long_format: flags.long || config.long_format,
            human_readable: flags.human_readable || config.human_readable,
            no_icons: flags.no_icons || config.no_icons,
            gitignore: flags.gitignore || config.gitignore,
            fuzzy_time: flags.fuzzy_time || config.fuzzy_time,
        }
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
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_default_params() {
        let params = Params::default();
        assert!(!params.show_all);
        assert!(!params.append_slash);
        assert!(!params.dirs_first);
        assert!(!params.almost_all);
        assert!(!params.long_format);
        assert!(!params.human_readable);
        assert!(!params.no_icons);
        assert!(!params.gitignore);
        assert!(!params.fuzzy_time);
    }

    #[test]
    fn test_config_conversion() -> std::io::Result<()> {
        let temp_dir = tempdir()?;
        let config_path = temp_dir.path().join("config.toml");

        let config_content = r#"
            show_all = true
            append_slash = true
            dirs_first = true
            long_format = true
            human_readable = true
            gitignore = true
        "#;

        fs::write(&config_path, config_content)?;

        let config = config::Config::builder()
            .add_source(config::File::from(config_path))
            .build()
            .unwrap();

        let params: Params = config.into();

        assert!(params.show_all);
        assert!(params.append_slash);
        assert!(params.dirs_first);
        assert!(params.long_format);
        assert!(params.human_readable);
        assert!(params.gitignore);

        Ok(())
    }

    #[test]
    fn test_params_merge() {
        let config = Params {
            show_all: true,
            append_slash: true,
            dirs_first: false,
            almost_all: false,
            long_format: true,
            human_readable: true,
            no_icons: false,
            gitignore: true,
            fuzzy_time: false,
        };

        let flags = cli::Flags {
            version: false,
            paths: vec![],
            show_all: false,
            almost_all: true,
            slash: false,
            dirs_first: true,
            long: false,
            human_readable: false,
            no_icons: true,
            gitignore: false,
            fuzzy_time: true,
        };

        let params = Params::merge(&flags, &config);

        assert!(params.show_all);
        assert!(params.append_slash);
        assert!(params.dirs_first);
        assert!(params.almost_all);
        assert!(params.long_format);
        assert!(params.human_readable);
        assert!(params.no_icons);
        assert!(params.gitignore);
        assert!(params.fuzzy_time);
    }
}
