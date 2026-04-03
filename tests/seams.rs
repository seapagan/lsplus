use lsplus::Params;
use lsplus::app::run_with_flags;
use lsplus::cli::Flags;
use lsplus::utils::file::create_file_info;
use std::fs;
use tempfile::tempdir;

#[cfg(unix)]
#[test]
fn test_public_create_file_info_formats_short_symlinks() {
    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().join("target.txt");
    let link = temp_dir.path().join("link");

    fs::write(&target, "target").unwrap();
    std::os::unix::fs::symlink(&target, &link).unwrap();

    let short = create_file_info(&link, &Params::default()).unwrap();
    assert!(short.display_name.contains("link"));
    assert!(!short.display_name.contains('*'));

    let short_with_marker = create_file_info(
        &link,
        &Params {
            append_slash: true,
            ..Params::default()
        },
    )
    .unwrap();
    assert!(short_with_marker.display_name.contains('*'));
}

#[test]
fn test_public_run_with_flags_accepts_missing_patterns() {
    let flags = Flags {
        show_all: false,
        almost_all: false,
        long: false,
        human_readable: false,
        paths: vec![String::from("**/definitely_missing_coverage_pattern")],
        slash: false,
        dirs_first: false,
        no_icons: false,
        gitignore: false,
        version: false,
        fuzzy_time: false,
    };

    assert!(run_with_flags(flags).is_ok());
}
