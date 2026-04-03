use config::Config;
use lsplus::Params;
use lsplus::cli::Flags;
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
fn test_config_conversion() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    fs::write(
        &config_path,
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

    let config = Config::builder()
        .add_source(config::File::from(config_path))
        .build()
        .unwrap();

    let params: Params = config.into();

    assert_eq!(
        params,
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
}

#[test]
fn test_params_merge_prefers_true_from_either_source() {
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

    let flags = Flags {
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

#[test]
fn test_params_merge_keeps_false_when_both_sources_are_false() {
    let flags = Flags {
        version: false,
        paths: vec![],
        show_all: false,
        almost_all: false,
        slash: false,
        dirs_first: false,
        long: false,
        human_readable: false,
        no_icons: false,
        gitignore: false,
        fuzzy_time: false,
    };

    let params = Params::merge(&flags, &Params::default());

    assert_eq!(params, Params::default());
}
