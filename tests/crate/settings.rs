use crate::Params;
use crate::settings::{
    config_path_from_home, load_config, load_config_from_path,
};
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_load_config_returns_default_when_path_is_missing() {
    assert_eq!(load_config_from_path(None), Params::default());
}

#[test]
fn test_config_path_from_home_handles_some_and_none() {
    assert_eq!(config_path_from_home(None), None);
    assert_eq!(
        config_path_from_home(Some(PathBuf::from("/tmp/home"))),
        Some(PathBuf::from("/tmp/home/.config/lsplus/config.toml"))
    );
}

#[test]
fn test_load_config_returns_default_when_config_is_missing() {
    let temp_dir = tempdir().unwrap();

    temp_env::with_var("HOME", Some(temp_dir.path()), || {
        assert_eq!(load_config(), Params::default());
    });
}

#[test]
fn test_load_config_returns_default_when_config_is_invalid() {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().join(".config").join("lsplus");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(config_dir.join("config.toml"), "invalid = toml [ content")
        .unwrap();

    temp_env::with_var("HOME", Some(temp_dir.path()), || {
        assert_eq!(load_config(), Params::default());
    });
}

#[test]
fn test_load_config_reads_boolean_settings_from_home_config() {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().join(".config").join("lsplus");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("config.toml"),
        r#"
            show_all = true
            append_slash = true
            dirs_first = true
            almost_all = true
            long_format = true
            human_readable = true
            no_icons = true
            gitignore = true
            fuzzy_time = true
        "#,
    )
    .unwrap();

    temp_env::with_var("HOME", Some(temp_dir.path()), || {
        assert_eq!(
            load_config(),
            Params {
                show_all: true,
                append_slash: true,
                dirs_first: true,
                almost_all: true,
                long_format: true,
                human_readable: true,
                no_icons: true,
                gitignore: true,
                fuzzy_time: true,
            }
        );
    });
}
