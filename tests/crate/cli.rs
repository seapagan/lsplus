use crate::IndicatorStyle;
use crate::cli::{
    CompatMode, Flags, format_version_info, try_parse_from_mode, version_info,
};

#[test]
fn test_default_flags() {
    let args = Flags::parse_from(["lsplus"]);
    assert!(!args.show_all);
    assert!(!args.almost_all);
    assert!(!args.long);
    assert!(!args.human_readable);
    assert_eq!(args.indicator_style, None);
    assert!(!args.dirs_first);
    assert!(!args.no_icons);
    assert!(!args.no_color);
    assert!(!args.gitignore);
    assert!(!args.version);
    assert!(!args.fuzzy_time);
    assert_eq!(args.paths, vec![String::from(".")]);
}

#[test]
fn test_multiple_paths() {
    let args = Flags::parse_from(["lsplus", "path1", "path2"]);
    assert_eq!(
        args.paths,
        vec![String::from("path1"), String::from("path2")]
    );
}

#[test]
fn test_all_flags() {
    let args = Flags::parse_from([
        "lsplus",
        "-a",
        "-A",
        "-l",
        "-h",
        "-p",
        "--sort-dirs",
        "--no-icons",
        "--no-color",
        "--gitignore",
        "--fuzzy-time",
    ]);
    assert!(args.show_all);
    assert!(args.almost_all);
    assert!(args.long);
    assert!(args.human_readable);
    assert_eq!(args.indicator_style, Some(IndicatorStyle::Slash));
    assert!(args.dirs_first);
    assert!(args.no_icons);
    assert!(args.no_color);
    assert!(args.gitignore);
    assert!(args.fuzzy_time);
}

#[test]
fn test_no_color_short_flag() {
    let args = Flags::parse_from(["lsplus", "-N"]);
    assert!(args.no_color);
}

#[test]
fn test_version_flag() {
    let args = Flags::parse_from(["lsplus", "--version"]);
    assert!(args.version);
}

#[test]
fn test_version_info_uses_package_metadata() {
    let info = version_info();
    assert!(info.contains("lsplus v"));
    assert!(info.contains("Released under the MIT license by"));
    assert!(info.contains(env!("CARGO_PKG_AUTHORS")));
    assert!(info.contains(env!("CARGO_PKG_DESCRIPTION")));
}

#[test]
fn test_format_version_info_uses_fallback_values() {
    let info = format_version_info("1.2.3", "", "");

    assert!(info.contains("lsplus v1.2.3"));
    assert!(info.contains("No description provided"));
    assert!(info.contains("Released under the MIT license by Unknown"));
}

#[test]
fn test_format_version_info_preserves_supplied_values() {
    let info = format_version_info("1.2.3", "Grant Ramsay", "Lists files");

    assert!(info.contains("lsplus v1.2.3"));
    assert!(info.contains("Lists files"));
    assert!(info.contains("Grant Ramsay"));
}

#[test]
fn test_help_flag() {
    let result = Flags::try_parse_from(["lsplus", "--help"]);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Usage:"));
}

#[test]
fn test_parse_from_mode_native_keeps_conflicting_short_flags() {
    let args = try_parse_from_mode(
        CompatMode::Native,
        ["lsplus", "-D", "-I", "-N", "-Z"],
    )
    .unwrap();

    assert!(args.dirs_first);
    assert!(args.gitignore);
    assert!(args.no_color);
    assert!(args.fuzzy_time);
}

#[test]
fn test_parse_from_mode_gnu_rejects_conflicting_short_flags() {
    for flag in ["-D", "-I", "-N", "-Z"] {
        let err = try_parse_from_mode(CompatMode::Gnu, ["lsplus", flag])
            .unwrap_err();

        assert!(err.to_string().contains("unexpected argument"));
        assert!(err.to_string().contains(flag));
    }
}

#[test]
fn test_parse_from_mode_gnu_accepts_long_options_for_conflicts() {
    let args = try_parse_from_mode(
        CompatMode::Gnu,
        [
            "lsplus",
            "--indicator-style=slash",
            "--group-directories-first",
            "--gitignore",
            "--no-color",
            "--fuzzy-time",
        ],
    )
    .unwrap();

    assert_eq!(args.indicator_style, Some(IndicatorStyle::Slash));
    assert!(args.dirs_first);
    assert!(args.gitignore);
    assert!(args.no_color);
    assert!(args.fuzzy_time);
}

#[test]
fn test_parse_from_mode_gnu_help_omits_conflicting_short_flags() {
    let err = try_parse_from_mode(CompatMode::Gnu, ["lsplus", "--help"])
        .unwrap_err();
    let help = err.to_string();

    assert!(help.contains("-p"));
    assert!(help.contains("--file-type"));
    assert!(help.contains("-F"));
    assert!(help.contains("--indicator-style"));
    assert!(help.contains("--group-directories-first"));
    assert!(!help.contains("--slash-dirs"));
    assert!(!help.contains("-D,"));
    assert!(!help.contains("-I,"));
    assert!(!help.contains("-N,"));
    assert!(!help.contains("-Z,"));
}

#[test]
fn test_parse_from_mode_gnu_short_p_sets_slash() {
    let args = try_parse_from_mode(CompatMode::Gnu, ["lsplus", "-p"]).unwrap();

    assert_eq!(args.indicator_style, Some(IndicatorStyle::Slash));
}

#[test]
fn test_parse_from_mode_native_accepts_file_type_and_classify_options() {
    let file_type =
        try_parse_from_mode(CompatMode::Native, ["lsplus", "--file-type"])
            .unwrap();
    let classify =
        try_parse_from_mode(CompatMode::Native, ["lsplus", "-F"]).unwrap();
    let no_indicators =
        try_parse_from_mode(CompatMode::Native, ["lsplus", "--no-indicators"])
            .unwrap();

    assert_eq!(file_type.indicator_style, Some(IndicatorStyle::FileType));
    assert_eq!(classify.indicator_style, Some(IndicatorStyle::Classify));
    assert_eq!(no_indicators.indicator_style, Some(IndicatorStyle::None));
}

#[test]
fn test_parse_from_mode_gnu_accepts_indicator_style_variants() {
    let file_type = try_parse_from_mode(
        CompatMode::Gnu,
        ["lsplus", "--indicator-style=file-type"],
    )
    .unwrap();
    let classify = try_parse_from_mode(
        CompatMode::Gnu,
        ["lsplus", "--indicator-style=classify"],
    )
    .unwrap();
    let none = try_parse_from_mode(
        CompatMode::Gnu,
        ["lsplus", "--indicator-style=none"],
    )
    .unwrap();

    assert_eq!(file_type.indicator_style, Some(IndicatorStyle::FileType));
    assert_eq!(classify.indicator_style, Some(IndicatorStyle::Classify));
    assert_eq!(none.indicator_style, Some(IndicatorStyle::None));
}

#[test]
fn test_parse_from_mode_rejects_conflicting_indicator_options() {
    let native_err = try_parse_from_mode(
        CompatMode::Native,
        ["lsplus", "-p", "--file-type"],
    )
    .unwrap_err();
    let gnu_err = try_parse_from_mode(
        CompatMode::Gnu,
        ["lsplus", "-F", "--indicator-style=file-type"],
    )
    .unwrap_err();

    assert!(native_err.to_string().contains("cannot be used"));
    assert!(gnu_err.to_string().contains("cannot be used"));
}

#[test]
fn test_parse_from_mode_gnu_rejects_native_slash_dirs_long_option() {
    let err = try_parse_from_mode(CompatMode::Gnu, ["lsplus", "--slash-dirs"])
        .unwrap_err();

    assert!(err.to_string().contains("unexpected argument"));
    assert!(err.to_string().contains("--slash-dirs"));
}

#[test]
fn test_parse_from_mode_gnu_rejects_native_classify_and_no_indicators() {
    for flag in ["--classify", "--no-indicators"] {
        let err = try_parse_from_mode(CompatMode::Gnu, ["lsplus", flag])
            .unwrap_err();

        assert!(err.to_string().contains("unexpected argument"));
        assert!(err.to_string().contains(flag));
    }
}

#[test]
fn test_parse_from_mode_gnu_rejects_native_sort_dirs_long_option() {
    let err = try_parse_from_mode(CompatMode::Gnu, ["lsplus", "--sort-dirs"])
        .unwrap_err();

    assert!(err.to_string().contains("unexpected argument"));
    assert!(err.to_string().contains("--sort-dirs"));
}
