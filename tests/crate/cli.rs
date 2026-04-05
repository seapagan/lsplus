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
    assert!(!args.slash);
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
    assert!(args.slash);
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

    assert!(args.slash);
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

    assert!(args.slash);
}

#[test]
fn test_parse_from_mode_gnu_rejects_native_slash_dirs_long_option() {
    let err = try_parse_from_mode(CompatMode::Gnu, ["lsplus", "--slash-dirs"])
        .unwrap_err();

    assert!(err.to_string().contains("unexpected argument"));
    assert!(err.to_string().contains("--slash-dirs"));
}

#[test]
fn test_parse_from_mode_gnu_rejects_native_sort_dirs_long_option() {
    let err = try_parse_from_mode(CompatMode::Gnu, ["lsplus", "--sort-dirs"])
        .unwrap_err();

    assert!(err.to_string().contains("unexpected argument"));
    assert!(err.to_string().contains("--sort-dirs"));
}
