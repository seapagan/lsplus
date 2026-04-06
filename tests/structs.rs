use config::Config;
use lsplus::cli::Flags;
use lsplus::{IndicatorStyle, Params};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_default_params() {
    let params = Params::default();
    assert!(!params.show_all);
    assert_eq!(params.indicator_style, IndicatorStyle::None);
    assert!(!params.dirs_first);
    assert!(!params.almost_all);
    assert!(!params.long_format);
    assert!(!params.human_readable);
    assert!(!params.no_icons);
    assert!(!params.no_color);
    assert!(!params.gitignore);
    assert!(!params.fuzzy_time);
}

#[test]
fn test_config_conversion() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    fs::write(
        &config_path,
        r#"
            show_all = true
            indicator_style = "file-type"
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

    let config = Config::builder()
        .add_source(config::File::from(config_path))
        .build()
        .unwrap();

    let params: Params = config.into();

    assert_eq!(
        params,
        Params {
            show_all: true,
            indicator_style: IndicatorStyle::FileType,
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
}

#[test]
fn test_config_conversion_maps_append_slash_alias_to_slash_style() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    fs::write(&config_path, "append_slash = true\n").unwrap();

    let config = Config::builder()
        .add_source(config::File::from(config_path))
        .build()
        .unwrap();

    let params: Params = config.into();

    assert_eq!(params.indicator_style, IndicatorStyle::Slash);
}

#[test]
fn test_config_conversion_prefers_indicator_style_over_append_slash_alias() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    fs::write(
        &config_path,
        "append_slash = true\nindicator_style = \"classify\"\n",
    )
    .unwrap();

    let config = Config::builder()
        .add_source(config::File::from(config_path))
        .build()
        .unwrap();

    let params: Params = config.into();

    assert_eq!(params.indicator_style, IndicatorStyle::Classify);
}

#[test]
fn test_params_merge_prefers_true_from_either_source() {
    let config = Params {
        show_all: true,
        indicator_style: IndicatorStyle::FileType,
        dirs_first: false,
        almost_all: false,
        long_format: true,
        human_readable: true,
        no_icons: false,
        no_color: true,
        gitignore: true,
        fuzzy_time: false,
    };

    let flags = Flags {
        version: false,
        paths: vec![],
        show_all: false,
        almost_all: true,
        indicator_style: Some(IndicatorStyle::Classify),
        dirs_first: true,
        long: false,
        human_readable: false,
        no_icons: true,
        no_color: false,
        gitignore: false,
        fuzzy_time: true,
    };

    let params = Params::merge(&flags, &config);

    assert!(params.show_all);
    assert_eq!(params.indicator_style, IndicatorStyle::Classify);
    assert!(params.dirs_first);
    assert!(params.almost_all);
    assert!(params.long_format);
    assert!(params.human_readable);
    assert!(params.no_icons);
    assert!(params.no_color);
    assert!(params.gitignore);
    assert!(params.fuzzy_time);
}

#[test]
fn test_params_merge_keeps_false_when_both_sources_are_false() {
    let flags = Flags {
        version: false,
        paths: vec![],
        show_all: false,
        almost_all: false,
        indicator_style: None,
        dirs_first: false,
        long: false,
        human_readable: false,
        no_icons: false,
        no_color: false,
        gitignore: false,
        fuzzy_time: false,
    };

    let params = Params::merge(&flags, &Params::default());

    assert_eq!(params, Params::default());
}
