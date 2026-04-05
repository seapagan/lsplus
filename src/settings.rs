use std::path::PathBuf;

use config::{Config, File, FileFormat};
use dirs_next::home_dir;
use serde::Deserialize;

use crate::Params;
use crate::cli::CompatMode;

pub const COMPAT_MODE_ENV_VAR: &str = "LSP_COMPAT_MODE";

#[derive(Debug, PartialEq)]
pub struct StartupConfig {
    pub params: Params,
    pub compat_mode: CompatMode,
}

#[derive(Debug, Deserialize, PartialEq, Default)]
struct ParsedConfig {
    #[serde(default, flatten)]
    params: Params,
    compat_mode: Option<String>,
}

fn config_path() -> Option<PathBuf> {
    config_path_from_home(home_dir())
}

pub(crate) fn config_path_from_home(home: Option<PathBuf>) -> Option<PathBuf> {
    let mut path = home?;
    path.push(".config/lsplus/config.toml");
    Some(path)
}

pub fn load_config() -> Params {
    load_config_from_path(config_path())
}

pub(crate) fn load_config_from_path(config_path: Option<PathBuf>) -> Params {
    load_parsed_config_from_path(config_path)
        .map(|config| config.params)
        .unwrap_or_default()
}

pub fn load_startup_config() -> Result<StartupConfig, String> {
    load_startup_config_from(
        config_path(),
        std::env::var(COMPAT_MODE_ENV_VAR).ok(),
    )
}

pub(crate) fn load_startup_config_from(
    config_path: Option<PathBuf>,
    env_mode: Option<String>,
) -> Result<StartupConfig, String> {
    let parsed_config = load_parsed_config_from_path(config_path);
    let compat_mode =
        resolve_compat_mode(env_mode.as_deref(), parsed_config.as_ref())?;

    Ok(StartupConfig {
        params: parsed_config
            .map(|config| config.params)
            .unwrap_or_default(),
        compat_mode,
    })
}

fn load_parsed_config_from_path(
    config_path: Option<PathBuf>,
) -> Option<ParsedConfig> {
    let config_path = config_path?;

    if !config_path.is_file() {
        return None;
    }

    let settings = Config::builder()
        .add_source(File::from(config_path).format(FileFormat::Toml))
        .build();

    match settings {
        Ok(config) => match config.try_deserialize::<ParsedConfig>() {
            Ok(parsed_config) => Some(parsed_config),
            Err(e) => {
                eprintln!("Error loading config: {}", e);
                None
            }
        },
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            None
        }
    }
}

fn resolve_compat_mode(
    env_mode: Option<&str>,
    parsed_config: Option<&ParsedConfig>,
) -> Result<CompatMode, String> {
    if let Some(mode) = env_mode {
        return CompatMode::parse_value(mode).map_err(|err| {
            format!("invalid {} value: {}", COMPAT_MODE_ENV_VAR, err)
        });
    }

    if let Some(mode) =
        parsed_config.and_then(|config| config.compat_mode.as_deref())
    {
        return CompatMode::parse_value(mode)
            .map_err(|err| format!("invalid compat_mode setting: {}", err));
    }

    Ok(CompatMode::Native)
}
