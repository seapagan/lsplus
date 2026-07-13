use crate::cli::CompatMode;
#[cfg(unix)]
use crate::settings::config_path_from_home;
#[cfg(unix)]
use crate::settings::load_config;
use crate::settings::{
    StartupConfig, load_config_from_path, load_startup_config_from,
    resolve_config_path,
};
#[cfg(unix)]
use crate::{
    IndicatorStyle,
    structs::{AttributeDisplay, PermissionDisplay},
};
use crate::{Params, ShortFormat};
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_load_config_returns_default_when_path_is_missing() {
    assert_eq!(load_config_from_path(None), Params::default());
}

#[test]
fn test_config_path_override_precedes_default() {
    let default = PathBuf::from("default.toml");

    assert_eq!(
        resolve_config_path(
            Some(OsString::from("override.toml")),
            Some(default.clone()),
        ),
        Some(PathBuf::from("override.toml"))
    );
    assert_eq!(
        resolve_config_path(None, Some(default.clone())),
        Some(default.clone())
    );
    assert_eq!(
        resolve_config_path(Some(OsString::new()), Some(default.clone())),
        Some(default)
    );
}

#[test]
#[cfg(unix)]
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
    let config_path = temp_dir.path().join("config.toml");

    assert_eq!(load_config_from_path(Some(config_path)), Params::default());
}

#[test]
fn test_load_config_returns_default_when_config_is_invalid() {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().join(".config").join("lsplus");
    fs::create_dir_all(&config_dir).unwrap();
    let config_path = config_dir.join("config.toml");
    fs::write(&config_path, "invalid = toml [ content").unwrap();

    assert_eq!(load_config_from_path(Some(config_path)), Params::default());
}

#[test]
fn test_load_config_returns_default_when_deserialization_fails() {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().join(".config").join("lsplus");
    fs::create_dir_all(&config_dir).unwrap();
    let config_path = config_dir.join("config.toml");
    fs::write(&config_path, "show_all = \"yes\"\n").unwrap();

    assert_eq!(load_config_from_path(Some(config_path)), Params::default());
}

#[test]
fn test_load_config_reads_vertical_short_format() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    fs::write(&config_path, "short_format = \"vertical\"\n").unwrap();

    let params = load_config_from_path(Some(config_path));

    assert_eq!(params.short_format, Some(ShortFormat::Vertical));
}

#[test]
fn test_load_config_reads_icon_display() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    fs::write(&config_path, "icons = \"always\"\n").unwrap();

    let params = load_config_from_path(Some(config_path));

    assert_eq!(params.icons, crate::IconDisplay::Always);
}

#[test]
#[cfg(unix)]
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
            header = true
            human_readable = true
            no_icons = true
            no_color = true
            permission_colors = false
            time_gradient = false
            size_colors = false
            gitignore = true
            prune_noisy_dirs = true
            prune_dirs = ["target", "dist"]
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
                short_format: None,
                header: true,
                human_readable: true,
                si: false,
                recursive: false,
                tree: false,
                tree_level: 2,
                recursive_level: None,
                prune_dirs: vec![
                    String::from("target"),
                    String::from("dist"),
                    String::from(".git"),
                    String::from(".hg"),
                    String::from(".svn"),
                    String::from("node_modules"),
                    String::from("__pycache__"),
                ],
                icons: crate::IconDisplay::Auto,
                no_icons: true,
                no_color: true,
                permission_colors: false,
                permissions: PermissionDisplay::Symbolic,
                attributes: AttributeDisplay::Long,
                time_gradient: false,
                size_colors: false,
                gitignore: true,
                fuzzy_time: true,
            }
        );
    });
}

#[test]
#[cfg(unix)]
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
#[cfg(unix)]
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
