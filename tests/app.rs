use lsplus::Params;
use lsplus::app::{collect_matches, patterns_from_args, run_with_flags};
use lsplus::cli::Flags;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_patterns_from_args_defaults_to_current_directory() {
    assert_eq!(patterns_from_args(Vec::new()), vec![String::from(".")]);
}

#[test]
fn test_patterns_from_args_preserves_explicit_paths() {
    let paths = vec![String::from("left"), String::from("right")];

    assert_eq!(patterns_from_args(paths.clone()), paths);
}

#[test]
fn test_collect_matches_filters_glob_results() {
    let temp_dir = tempdir().unwrap();
    fs::write(temp_dir.path().join("test1.txt"), "one").unwrap();
    fs::write(temp_dir.path().join("test2.txt"), "two").unwrap();
    fs::write(temp_dir.path().join("test.rs"), "fn main() {}").unwrap();

    let pattern = format!("{}/*.txt", temp_dir.path().display());
    let matches = collect_matches(&[pattern], &Params::default()).unwrap();

    assert_eq!(matches.len(), 2);
    assert!(
        matches
            .iter()
            .all(|info| info.display_name.contains(".txt"))
    );
}

#[test]
fn test_collect_matches_returns_empty_for_missing_pattern() {
    let matches = collect_matches(
        &[String::from("**/nonexistent_pattern_*.xyz")],
        &Params::default(),
    )
    .unwrap();

    assert!(matches.is_empty());
}

#[test]
fn test_collect_matches_handles_invalid_glob() {
    let matches = collect_matches(
        &[String::from("[invalid-glob-pattern")],
        &Params::default(),
    )
    .unwrap();

    assert!(matches.is_empty());
}

#[test]
fn test_run_with_flags_lists_matching_entries() {
    let temp_dir = tempdir().unwrap();
    fs::write(temp_dir.path().join("alpha.txt"), "alpha").unwrap();
    fs::write(temp_dir.path().join("beta.txt"), "beta").unwrap();

    temp_env::with_var("HOME", Some(temp_dir.path()), || {
        let flags = Flags {
            show_all: false,
            almost_all: false,
            long: false,
            human_readable: false,
            paths: vec![temp_dir.path().display().to_string()],
            slash: false,
            dirs_first: false,
            no_icons: true,
            gitignore: false,
            version: false,
            fuzzy_time: false,
        };

        assert!(run_with_flags(flags).is_ok());
    });
}
