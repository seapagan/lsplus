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

#[derive(Debug, Deserialize, PartialEq, Default)]
pub struct Params {
    pub show_all: bool,
    pub append_slash: bool,
    pub dirs_first: bool,
    pub almost_all: bool,
    pub long_format: bool,
    pub human_readable: bool,
    pub no_icons: bool,
    pub fuzzy_time: bool,
    pub shorten_names: bool,
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
            fuzzy_time,
            shorten_names
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
        assert!(!params.fuzzy_time);
        assert!(!params.shorten_names);
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

        Ok(())
    }
}
