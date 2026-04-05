use crate::cli::CompatMode;
use crate::settings::{
    StartupConfig, config_path_from_home, load_config, load_config_from_path,
    load_startup_config_from,
};
use crate::{IndicatorStyle, Params};
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
            indicator_style = "classify"
            dirs_first = true
            almost_all = true
            long_format = true
            human_readable = true
            no_icons = true
            no_color = true
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
                indicator_style: IndicatorStyle::Classify,
                dirs_first: true,
                almost_all: true,
                long_format: true,
                human_readable: true,
                no_icons: true,
                no_color: true,
                gitignore: true,
                fuzzy_time: true,
            }
        );
    });
}

#[test]
fn test_load_config_maps_append_slash_alias_to_indicator_style() {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().join(".config").join("lsplus");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(config_dir.join("config.toml"), "append_slash = true\n")
        .unwrap();

    temp_env::with_var("HOME", Some(temp_dir.path()), || {
        assert_eq!(
            load_config(),
            Params {
                indicator_style: IndicatorStyle::Slash,
                ..Params::default()
            }
        );
    });
}

#[test]
fn test_load_config_prefers_indicator_style_over_append_slash_alias() {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().join(".config").join("lsplus");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("config.toml"),
        "append_slash = true\nindicator_style = \"file-type\"\n",
    )
    .unwrap();

    temp_env::with_var("HOME", Some(temp_dir.path()), || {
        assert_eq!(
            load_config(),
            Params {
                indicator_style: IndicatorStyle::FileType,
                ..Params::default()
            }
        );
    });
}

#[test]
fn test_load_startup_config_defaults_to_native_without_sources() {
    assert_eq!(
        load_startup_config_from(None, None).unwrap(),
        StartupConfig {
            params: Params::default(),
            compat_mode: CompatMode::Native,
        }
    );
}

#[test]
fn test_load_startup_config_reads_compat_mode_from_config() {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().join(".config").join("lsplus");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("config.toml"),
        r#"
            compat_mode = "gnu"
            no_color = true
        "#,
    )
    .unwrap();

    assert_eq!(
        load_startup_config_from(Some(config_dir.join("config.toml")), None,)
            .unwrap(),
        StartupConfig {
            params: Params {
                no_color: true,
                ..Params::default()
            },
            compat_mode: CompatMode::Gnu,
        }
    );
}

#[test]
fn test_load_startup_config_env_overrides_config_mode() {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().join(".config").join("lsplus");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("config.toml"),
        r#"
            compat_mode = "native"
            fuzzy_time = true
        "#,
    )
    .unwrap();

    let startup = load_startup_config_from(
        Some(config_dir.join("config.toml")),
        Some(String::from("gnu")),
    )
    .unwrap();

    assert_eq!(startup.compat_mode, CompatMode::Gnu);
    assert!(startup.params.fuzzy_time);
}

#[test]
fn test_load_startup_config_rejects_invalid_env_mode() {
    let err = load_startup_config_from(None, Some(String::from("bogus")))
        .unwrap_err();

    assert!(err.contains("LSP_COMPAT_MODE"));
    assert!(err.contains("bogus"));
}

#[test]
fn test_load_startup_config_rejects_invalid_config_mode() {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().join(".config").join("lsplus");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("config.toml"),
        r#"
            compat_mode = "bogus"
        "#,
    )
    .unwrap();

    let err =
        load_startup_config_from(Some(config_dir.join("config.toml")), None)
            .unwrap_err();

    assert!(err.contains("compat_mode"));
    assert!(err.contains("bogus"));
}
