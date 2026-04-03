use crate::cli::{Flags, format_version_info, version_info};
use clap::Parser;

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
    assert!(args.gitignore);
    assert!(args.fuzzy_time);
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
