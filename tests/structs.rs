use config::Config;
use lsplus::cli::Flags;
use lsplus::utils::format::SizeScale;
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
    assert!(!params.si);
    assert!(!params.recursive);
    assert!(!params.tree);
    assert_eq!(params.tree_level, 2);
    assert_eq!(params.recursive_level, None);
    assert!(!params.prune_noisy_dirs);
    assert!(params.prune_dirs.is_empty());
    assert_eq!(params.size_scale(), None);
    assert!(!params.no_icons);
    assert!(!params.no_color);
    assert!(params.permission_colors);
    assert!(params.time_gradient);
    assert!(params.size_colors);
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
            si = true
            recursive = true
            tree = true
            tree_level = 4
            prune_noisy_dirs = true
            prune_dirs = ["target", "dist"]
            no_icons = true
            no_color = true
            permission_colors = false
            time_gradient = false
            size_colors = false
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
            si: true,
            recursive: true,
            tree: true,
            tree_level: 4,
            recursive_level: Some(4),
            prune_noisy_dirs: true,
            prune_dirs: vec![
                String::from("target"),
                String::from("dist"),
                String::from(".git"),
                String::from(".hg"),
                String::from(".svn"),
                String::from("node_modules"),
                String::from("__pycache__"),
            ],
            no_icons: true,
            no_color: true,
            permission_colors: false,
            time_gradient: false,
            size_colors: false,
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
fn test_config_conversion_rejects_zero_tree_level() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    fs::write(&config_path, "tree_level = 0\n").unwrap();

    let config = Config::builder()
        .add_source(config::File::from(config_path))
        .build()
        .unwrap();

    let params: Params = config.into();

    assert_eq!(params.tree_level, 2);
    assert_eq!(params.recursive_level, None);
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
        si: false,
        recursive: true,
        tree: false,
        tree_level: 5,
        recursive_level: Some(5),
        prune_noisy_dirs: true,
        prune_dirs: vec![
            String::from(".git"),
            String::from(".hg"),
            String::from(".svn"),
            String::from("node_modules"),
            String::from("__pycache__"),
            String::from("from-config"),
        ],
        no_icons: false,
        no_color: true,
        permission_colors: true,
        time_gradient: false,
        size_colors: true,
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
        si: false,
        recursive: false,
        tree: true,
        tree_level: Some(3),
        prune_noisy_dirs: false,
        prune_dirs: vec![String::from("from-cli")],
        no_icons: true,
        no_color: false,
        no_permission_colors: true,
        no_time_gradient: false,
        no_size_colors: true,
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
    assert!(!params.si);
    assert!(params.recursive);
    assert!(params.tree);
    assert_eq!(params.tree_level, 3);
    assert_eq!(params.recursive_level, Some(3));
    assert!(params.prune_noisy_dirs);
    assert_eq!(
        params.prune_dirs,
        vec![
            String::from(".git"),
            String::from(".hg"),
            String::from(".svn"),
            String::from("node_modules"),
            String::from("__pycache__"),
            String::from("from-config"),
            String::from("from-cli"),
        ]
    );
    assert_eq!(params.size_scale(), Some(SizeScale::Binary));
    assert!(params.no_icons);
    assert!(params.no_color);
    assert!(!params.permission_colors);
    assert!(!params.time_gradient);
    assert!(!params.size_colors);
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
        si: false,
        recursive: false,
        tree: false,
        tree_level: None,
        prune_noisy_dirs: false,
        prune_dirs: Vec::new(),
        no_icons: false,
        no_color: false,
        no_permission_colors: false,
        no_time_gradient: false,
        no_size_colors: false,
        gitignore: false,
        fuzzy_time: false,
    };

    let params = Params::merge(&flags, &Params::default());

    assert_eq!(params, Params::default());
}

#[test]
fn test_params_merge_si_enables_decimal_human_readable_output() {
    let flags = Flags {
        version: false,
        paths: vec![],
        show_all: false,
        almost_all: false,
        indicator_style: None,
        dirs_first: false,
        long: false,
        human_readable: false,
        si: true,
        recursive: false,
        tree: false,
        tree_level: None,
        prune_noisy_dirs: false,
        prune_dirs: Vec::new(),
        no_icons: false,
        no_color: false,
        no_permission_colors: false,
        no_time_gradient: false,
        no_size_colors: false,
        gitignore: false,
        fuzzy_time: false,
    };

    let params = Params::merge(&flags, &Params::default());

    assert!(params.human_readable);
    assert!(params.si);
    assert_eq!(params.size_scale(), Some(SizeScale::Decimal));
}

#[test]
fn test_params_merge_config_si_overrides_config_human_readable() {
    let config = Params {
        human_readable: true,
        si: true,
        ..Params::default()
    };
    let flags = Flags {
        version: false,
        paths: vec![],
        show_all: false,
        almost_all: false,
        indicator_style: None,
        dirs_first: false,
        long: false,
        human_readable: false,
        si: false,
        recursive: false,
        tree: false,
        tree_level: None,
        prune_noisy_dirs: false,
        prune_dirs: Vec::new(),
        no_icons: false,
        no_color: false,
        no_permission_colors: false,
        no_time_gradient: false,
        no_size_colors: false,
        gitignore: false,
        fuzzy_time: false,
    };

    let params = Params::merge(&flags, &config);

    assert!(params.human_readable);
    assert!(params.si);
    assert_eq!(params.size_scale(), Some(SizeScale::Decimal));
}

#[test]
fn test_params_merge_cli_prune_dirs_append_config_prune_dirs() {
    let config = Params {
        prune_dirs: vec![String::from("from-config")],
        ..Params::default()
    };
    let flags = Flags {
        version: false,
        paths: vec![],
        show_all: false,
        almost_all: false,
        indicator_style: None,
        dirs_first: false,
        long: false,
        human_readable: false,
        si: false,
        recursive: false,
        tree: false,
        tree_level: None,
        prune_noisy_dirs: false,
        prune_dirs: vec![String::from("from-cli")],
        no_icons: false,
        no_color: false,
        no_permission_colors: false,
        no_time_gradient: false,
        no_size_colors: false,
        gitignore: false,
        fuzzy_time: false,
    };

    let params = Params::merge(&flags, &config);

    assert!(!params.prune_noisy_dirs);
    assert_eq!(
        params.prune_dirs,
        vec![String::from("from-config"), String::from("from-cli")]
    );
}
