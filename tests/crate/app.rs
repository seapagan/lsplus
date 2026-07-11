use crate::Params;
use crate::app::{
    collect_listing_sections, collect_tree_sections, patterns_from_args,
    run_with_flags, run_with_flags_and_config,
};
use crate::cli::Flags;
use crate::common_tests::ColorModeGuard;
use crate::utils::color::{color_mode_for, long_format_color_level};
use colored_text::{ColorLevel, ColorMode};
use std::fs;
use tempfile::tempdir;

fn default_flags_with_paths(paths: Vec<String>) -> Flags {
    Flags {
        show_all: false,
        almost_all: false,
        long: false,
        header: false,
        human_readable: false,
        si: false,
        recursive: false,
        tree: false,
        tree_level: None,
        prune_noisy_dirs: false,
        prune_dirs: Vec::new(),
        paths,
        indicator_style: None,
        dirs_first: false,
        no_icons: true,
        no_color: false,
        no_permission_colors: false,
        permissions: None,
        no_time_gradient: false,
        no_size_colors: false,
        gitignore: false,
        version: false,
        fuzzy_time: false,
    }
}

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
    let sections =
        collect_listing_sections(&[pattern], &Params::default()).unwrap();
    let matches = &sections[0].entries;

    assert_eq!(matches.len(), 2);
    assert!(
        matches
            .iter()
            .all(|info| info.display_name.contains(".txt"))
    );
}

#[test]
fn test_collect_matches_returns_empty_for_missing_pattern() {
    let sections = collect_listing_sections(
        &[String::from("**/nonexistent_pattern_*.xyz")],
        &Params::default(),
    )
    .unwrap();

    assert!(sections.is_empty());
}

#[test]
fn test_collect_listing_sections_returns_empty_for_empty_patterns() {
    let sections = collect_listing_sections(&[], &Params::default()).unwrap();

    assert!(sections.is_empty());
}

#[test]
fn test_collect_listing_sections_handles_invalid_glob() {
    let sections = collect_listing_sections(
        &[String::from("[invalid-glob-pattern")],
        &Params::default(),
    )
    .unwrap();

    assert!(sections.is_empty());
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
            header: false,
            human_readable: false,
            si: false,
            recursive: false,
            tree: false,
            tree_level: None,
            prune_noisy_dirs: false,
            prune_dirs: Vec::new(),
            paths: vec![temp_dir.path().display().to_string()],
            indicator_style: None,
            dirs_first: false,
            no_icons: true,
            no_color: false,
            no_permission_colors: false,
            permissions: None,
            no_time_gradient: false,
            no_size_colors: false,
            gitignore: false,
            version: false,
            fuzzy_time: false,
        };

        assert!(run_with_flags(flags).is_ok());
    });
}

#[test]
fn test_run_with_flags_and_config_rejects_tree_and_recursive_merge() {
    let flags = default_flags_with_paths(Vec::new());
    let config = Params {
        recursive: true,
        tree: true,
        ..Params::default()
    };

    let err = run_with_flags_and_config(flags, &config).unwrap_err();

    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    assert!(err.to_string().contains("--tree and --recursive"));
}

#[test]
fn test_run_with_flags_renders_multiple_long_directory_sections() {
    let temp_dir = tempdir().unwrap();
    let left = temp_dir.path().join("left");
    let right = temp_dir.path().join("right");
    fs::create_dir(&left).unwrap();
    fs::create_dir(&right).unwrap();
    fs::write(left.join("alpha.txt"), "alpha").unwrap();
    fs::write(right.join("beta.txt"), "beta").unwrap();

    let mut flags = default_flags_with_paths(vec![
        left.display().to_string(),
        right.display().to_string(),
    ]);
    flags.long = true;

    assert!(run_with_flags_and_config(flags, &Params::default()).is_ok());
}

#[test]
fn test_run_with_flags_renders_multiple_tree_sections() {
    let temp_dir = tempdir().unwrap();
    let left = temp_dir.path().join("left");
    let right = temp_dir.path().join("right");
    fs::create_dir(&left).unwrap();
    fs::create_dir(&right).unwrap();
    fs::write(left.join("alpha.txt"), "alpha").unwrap();
    fs::write(right.join("beta.txt"), "beta").unwrap();

    let mut flags = default_flags_with_paths(vec![
        left.display().to_string(),
        right.display().to_string(),
    ]);
    flags.tree = true;
    flags.long = true;

    assert!(run_with_flags_and_config(flags, &Params::default()).is_ok());
}

#[test]
fn test_run_with_flags_renders_recursive_long_sections() {
    let temp_dir = tempdir().unwrap();
    let nested = temp_dir.path().join("nested");
    fs::create_dir(&nested).unwrap();
    fs::write(temp_dir.path().join("root.txt"), "root").unwrap();
    fs::write(nested.join("deep.txt"), "deep").unwrap();

    let mut flags =
        default_flags_with_paths(vec![temp_dir.path().display().to_string()]);
    flags.recursive = true;
    flags.long = true;

    assert!(run_with_flags_and_config(flags, &Params::default()).is_ok());
}

#[test]
fn test_collect_listing_sections_groups_multiple_directories() {
    let temp_dir = tempdir().unwrap();
    let left = temp_dir.path().join("left");
    let right = temp_dir.path().join("right");
    fs::create_dir(&left).unwrap();
    fs::create_dir(&right).unwrap();
    fs::write(left.join("alpha.txt"), "alpha").unwrap();
    fs::write(right.join("beta.txt"), "beta").unwrap();

    let sections = collect_listing_sections(
        &[left.display().to_string(), right.display().to_string()],
        &Params::default(),
    )
    .unwrap();

    assert_eq!(sections.len(), 2);
    assert_eq!(sections[0].header, Some(left.display().to_string()));
    assert_eq!(sections[1].header, Some(right.display().to_string()));
    assert!(
        sections[0]
            .entries
            .iter()
            .any(|info| info.display_name.contains("alpha.txt"))
    );
    assert!(
        sections[1]
            .entries
            .iter()
            .any(|info| info.display_name.contains("beta.txt"))
    );
}

#[test]
fn test_collect_listing_sections_puts_files_before_directories() {
    let temp_dir = tempdir().unwrap();
    let file = temp_dir.path().join("top.txt");
    let dir = temp_dir.path().join("dir");
    fs::write(&file, "top").unwrap();
    fs::create_dir(&dir).unwrap();
    fs::write(dir.join("child.txt"), "child").unwrap();

    let sections = collect_listing_sections(
        &[dir.display().to_string(), file.display().to_string()],
        &Params::default(),
    )
    .unwrap();

    assert_eq!(sections.len(), 2);
    assert_eq!(sections[0].header, None);
    assert!(sections[0].entries[0].display_name.contains("top.txt"));
    assert_eq!(sections[1].header, Some(dir.display().to_string()));
}

#[test]
fn test_collect_listing_sections_keeps_single_directory_unlabeled() {
    let temp_dir = tempdir().unwrap();
    fs::write(temp_dir.path().join("child.txt"), "child").unwrap();

    let sections = collect_listing_sections(
        &[temp_dir.path().display().to_string()],
        &Params::default(),
    )
    .unwrap();

    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].header, None);
}

#[test]
fn test_collect_listing_sections_recurses_with_headers() {
    let temp_dir = tempdir().unwrap();
    let nested = temp_dir.path().join("nested");
    fs::create_dir(&nested).unwrap();
    fs::write(nested.join("deep.txt"), "deep").unwrap();
    let params = Params {
        recursive: true,
        ..Params::default()
    };

    let sections = collect_listing_sections(
        &[temp_dir.path().display().to_string()],
        &params,
    )
    .unwrap();

    assert_eq!(sections.len(), 2);
    assert_eq!(
        sections[0].header,
        Some(temp_dir.path().display().to_string())
    );
    assert_eq!(sections[1].header, Some(nested.display().to_string()));
    assert!(
        sections[1]
            .entries
            .iter()
            .any(|info| info.display_name.contains("deep.txt"))
    );
}

#[test]
fn test_collect_listing_sections_recursive_filters_glob_matches() {
    let temp_dir = tempdir().unwrap();
    let nested = temp_dir.path().join("nested");
    fs::create_dir(&nested).unwrap();
    fs::write(temp_dir.path().join("root.rs"), "root").unwrap();
    fs::write(temp_dir.path().join("root.txt"), "root").unwrap();
    fs::write(nested.join("deep.rs"), "deep").unwrap();
    fs::write(nested.join("deep.txt"), "deep").unwrap();
    let params = Params {
        recursive: true,
        ..Params::default()
    };
    let pattern = format!("{}/*.rs", temp_dir.path().display());

    let sections = collect_listing_sections(&[pattern], &params).unwrap();

    assert_eq!(sections.len(), 2);
    assert_eq!(
        sections[0].header,
        Some(temp_dir.path().display().to_string())
    );
    assert!(
        sections[0]
            .entries
            .iter()
            .any(|info| info.short_name == "root.rs")
    );
    assert!(
        !sections[0]
            .entries
            .iter()
            .any(|info| info.short_name == "root.txt")
    );
    assert_eq!(sections[1].header, Some(nested.display().to_string()));
    assert!(
        sections[1]
            .entries
            .iter()
            .any(|info| info.short_name == "deep.rs")
    );
    assert!(
        !sections[1]
            .entries
            .iter()
            .any(|info| info.short_name == "deep.txt")
    );
}

#[test]
fn test_collect_listing_sections_recursive_roots_prefixed_glob() {
    let temp_dir = tempdir().unwrap();
    let src = temp_dir.path().join("src");
    let utils = src.join("utils");
    fs::create_dir_all(&utils).unwrap();
    fs::write(temp_dir.path().join("outside.rs"), "outside").unwrap();
    fs::write(src.join("lib.rs"), "lib").unwrap();
    fs::write(src.join("lib.txt"), "lib").unwrap();
    fs::write(utils.join("file.rs"), "file").unwrap();
    let params = Params {
        recursive: true,
        ..Params::default()
    };
    let pattern = format!("{}/*.rs", src.display());

    let sections = collect_listing_sections(&[pattern], &params).unwrap();

    assert_eq!(sections.len(), 2);
    assert_eq!(sections[0].header, Some(src.display().to_string()));
    assert!(
        sections[0]
            .entries
            .iter()
            .any(|info| info.short_name == "lib.rs")
    );
    assert!(
        !sections[0]
            .entries
            .iter()
            .any(|info| info.short_name == "lib.txt")
    );
    assert_eq!(sections[1].header, Some(utils.display().to_string()));
    assert!(
        sections[1]
            .entries
            .iter()
            .any(|info| info.short_name == "file.rs")
    );
    assert!(
        !sections
            .iter()
            .flat_map(|section| section.entries.iter())
            .any(|info| info.short_name == "outside.rs")
    );
}

#[test]
fn test_collect_listing_sections_recursive_filter_no_matches_is_empty() {
    let temp_dir = tempdir().unwrap();
    fs::write(temp_dir.path().join("root.txt"), "root").unwrap();
    let params = Params {
        recursive: true,
        ..Params::default()
    };
    let pattern = format!("{}/*.rs", temp_dir.path().display());

    let sections = collect_listing_sections(&[pattern], &params).unwrap();

    assert!(sections.is_empty());
}

#[test]
fn test_collect_listing_sections_recursive_mixes_files_filters_and_directories()
 {
    let temp_dir = tempdir().unwrap();
    let top_file = temp_dir.path().join("top.txt");
    let src = temp_dir.path().join("src");
    let utils = src.join("utils");
    let docs = temp_dir.path().join("docs");
    fs::create_dir_all(&utils).unwrap();
    fs::create_dir(&docs).unwrap();
    fs::write(&top_file, "top").unwrap();
    fs::write(src.join("lib.rs"), "lib").unwrap();
    fs::write(src.join("lib.txt"), "lib").unwrap();
    fs::write(utils.join("file.rs"), "file").unwrap();
    fs::write(docs.join("guide.md"), "guide").unwrap();
    let params = Params {
        recursive: true,
        ..Params::default()
    };
    let pattern = format!("{}/*.rs", src.display());

    let sections = collect_listing_sections(
        &[
            pattern,
            top_file.display().to_string(),
            docs.display().to_string(),
        ],
        &params,
    )
    .unwrap();

    assert_eq!(sections.len(), 4);
    assert_eq!(sections[0].header, None);
    assert!(
        sections[0]
            .entries
            .iter()
            .any(|info| info.short_name == "top.txt")
    );
    assert_eq!(sections[1].header, Some(src.display().to_string()));
    assert!(
        sections[1]
            .entries
            .iter()
            .any(|info| info.short_name == "lib.rs")
    );
    assert!(
        !sections[1]
            .entries
            .iter()
            .any(|info| info.short_name == "lib.txt")
    );
    assert_eq!(sections[2].header, Some(utils.display().to_string()));
    assert!(
        sections[2]
            .entries
            .iter()
            .any(|info| info.short_name == "file.rs")
    );
    assert_eq!(sections[3].header, Some(docs.display().to_string()));
    assert!(
        sections[3]
            .entries
            .iter()
            .any(|info| info.short_name == "guide.md")
    );
}

#[test]
fn test_collect_listing_sections_recursive_missing_filter_root_is_empty() {
    let temp_dir = tempdir().unwrap();
    let params = Params {
        recursive: true,
        ..Params::default()
    };
    let pattern = format!("{}/missing/*.rs", temp_dir.path().display());

    let sections = collect_listing_sections(&[pattern], &params).unwrap();

    assert!(sections.is_empty());
}

#[test]
fn test_collect_listing_sections_recursive_invalid_filter_glob_is_empty() {
    let temp_dir = tempdir().unwrap();
    let params = Params {
        recursive: true,
        ..Params::default()
    };
    let pattern = format!("{}/[invalid", temp_dir.path().display());

    let sections = collect_listing_sections(&[pattern], &params).unwrap();

    assert!(sections.is_empty());
}

#[test]
fn test_collect_listing_sections_recursive_parent_glob_uses_operand_matches() {
    let temp_dir = tempdir().unwrap();
    let first = temp_dir.path().join("first");
    let second = temp_dir.path().join("second");
    fs::create_dir(&first).unwrap();
    fs::create_dir(&second).unwrap();
    fs::write(first.join("alpha.rs"), "alpha").unwrap();
    fs::write(second.join("beta.txt"), "beta").unwrap();
    let params = Params {
        recursive: true,
        ..Params::default()
    };
    let pattern = format!("{}/*/*.rs", temp_dir.path().display());

    let sections = collect_listing_sections(&[pattern], &params).unwrap();

    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].header, None);
    assert_eq!(sections[0].entries[0].short_name, "alpha.rs");
}

#[test]
fn test_collect_listing_sections_recursive_respects_level_limit() {
    let temp_dir = tempdir().unwrap();
    let child = temp_dir.path().join("child");
    let grandchild = child.join("grandchild");
    fs::create_dir(&child).unwrap();
    fs::create_dir(&grandchild).unwrap();
    fs::write(child.join("shown.txt"), "shown").unwrap();
    fs::write(grandchild.join("hidden.txt"), "hidden").unwrap();
    let params = Params {
        recursive: true,
        recursive_level: Some(2),
        ..Params::default()
    };

    let sections = collect_listing_sections(
        &[temp_dir.path().display().to_string()],
        &params,
    )
    .unwrap();

    assert_eq!(sections.len(), 2);
    assert_eq!(
        sections[0].header,
        Some(temp_dir.path().display().to_string())
    );
    assert_eq!(sections[1].header, Some(child.display().to_string()));
    assert!(
        sections[1]
            .entries
            .iter()
            .any(|info| info.display_name.contains("shown.txt"))
    );
    assert!(
        sections[1]
            .entries
            .iter()
            .any(|info| info.display_name.contains("grandchild"))
    );
    assert!(!sections.iter().any(
        |section| section.header == Some(grandchild.display().to_string())
    ));
}

#[test]
fn test_collect_listing_sections_recursive_level_one_shows_only_root_entries()
{
    let temp_dir = tempdir().unwrap();
    let child = temp_dir.path().join("child");
    let grandchild = child.join("grandchild");
    fs::create_dir(&child).unwrap();
    fs::create_dir(&grandchild).unwrap();
    fs::write(child.join("hidden.txt"), "hidden").unwrap();
    let params = Params {
        recursive: true,
        recursive_level: Some(1),
        ..Params::default()
    };

    let sections = collect_listing_sections(
        &[temp_dir.path().display().to_string()],
        &params,
    )
    .unwrap();

    assert_eq!(sections.len(), 1);
    assert_eq!(
        sections[0].header,
        Some(temp_dir.path().display().to_string())
    );
    assert!(
        sections[0]
            .entries
            .iter()
            .any(|info| info.display_name.contains("child"))
    );
    assert!(
        !sections[0]
            .entries
            .iter()
            .any(|info| info.display_name.contains("grandchild"))
    );
    assert!(!sections.iter().any(
        |section| section.header == Some(child.display().to_string())
    ));
    assert!(
        !sections
            .iter()
            .flat_map(|section| section.entries.iter())
            .any(|info| info.display_name.contains("hidden.txt"))
    );
}

#[test]
#[cfg(unix)]
fn test_collect_listing_sections_recursive_ignores_dot_entries() {
    let temp_dir = tempdir().unwrap();
    let nested = temp_dir.path().join("nested");
    fs::create_dir(&nested).unwrap();
    fs::write(nested.join(".hidden"), "hidden").unwrap();
    fs::write(nested.join("deep.txt"), "deep").unwrap();
    let params = Params {
        recursive: true,
        show_all: true,
        ..Params::default()
    };

    let sections = collect_listing_sections(
        &[temp_dir.path().display().to_string()],
        &params,
    )
    .unwrap();

    assert_eq!(sections.len(), 2);
    assert!(!sections.iter().any(|section| {
        section.header
            == Some(temp_dir.path().join("..").display().to_string())
    }));
    assert!(
        sections[0]
            .entries
            .iter()
            .any(|info| info.full_path == temp_dir.path().join("."))
    );
    assert!(
        sections[0]
            .entries
            .iter()
            .any(|info| info.full_path == temp_dir.path().join(".."))
    );

    let nested_section = sections
        .iter()
        .find(|section| section.header == Some(nested.display().to_string()))
        .unwrap();
    assert!(
        !nested_section
            .entries
            .iter()
            .any(|info| info.full_path == nested.join("."))
    );
    assert!(
        !nested_section
            .entries
            .iter()
            .any(|info| info.full_path == nested.join(".."))
    );
    assert!(
        nested_section
            .entries
            .iter()
            .any(|info| info.short_name == ".hidden")
    );
}

#[cfg(unix)]
#[test]
fn test_collect_listing_sections_recursive_skips_symlinked_directories() {
    let temp_dir = tempdir().unwrap();
    let real = temp_dir.path().join("real");
    let link = temp_dir.path().join("link");
    fs::create_dir(&real).unwrap();
    fs::write(real.join("deep.txt"), "deep").unwrap();
    std::os::unix::fs::symlink(&real, &link).unwrap();
    let params = Params {
        recursive: true,
        ..Params::default()
    };

    let sections = collect_listing_sections(
        &[temp_dir.path().display().to_string()],
        &params,
    )
    .unwrap();

    assert!(
        sections
            .iter()
            .any(|section| section.header == Some(real.display().to_string()))
    );
    assert!(
        !sections
            .iter()
            .any(|section| section.header == Some(link.display().to_string()))
    );
}

#[cfg(unix)]
#[test]
fn test_collect_listing_sections_lists_symlinked_directory_operand() {
    let temp_dir = tempdir().unwrap();
    let real = temp_dir.path().join("real");
    let link = temp_dir.path().join("link");
    fs::create_dir(&real).unwrap();
    fs::write(real.join("shown.txt"), "shown").unwrap();
    std::os::unix::fs::symlink(&real, &link).unwrap();
    let params = Params {
        recursive: true,
        ..Params::default()
    };

    let sections =
        collect_listing_sections(&[link.display().to_string()], &params)
            .unwrap();

    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].header, Some(link.display().to_string()));
    assert!(
        sections[0]
            .entries
            .iter()
            .any(|info| info.display_name.contains("shown.txt"))
    );
}

#[test]
fn test_collect_listing_sections_recursive_prunes_noisy_directories() {
    let temp_dir = tempdir().unwrap();
    let git_dir = temp_dir.path().join(".git");
    fs::create_dir(&git_dir).unwrap();
    fs::write(git_dir.join("config"), "config").unwrap();
    let params = Params {
        recursive: true,
        show_all: true,
        prune_dirs: vec![String::from(".git")],
        ..Params::default()
    };

    let sections = collect_listing_sections(
        &[temp_dir.path().display().to_string()],
        &params,
    )
    .unwrap();

    assert_eq!(sections.len(), 1);
    assert!(
        sections[0]
            .entries
            .iter()
            .any(|info| info.display_name.contains(".git"))
    );
    assert!(
        !sections
            .iter()
            .any(|section| section.header
                == Some(git_dir.display().to_string()))
    );
}

#[test]
fn test_collect_listing_sections_recursive_prunes_custom_directories() {
    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().join("target");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("hidden.txt"), "hidden").unwrap();
    let params = Params {
        recursive: true,
        prune_dirs: vec![String::from("target")],
        ..Params::default()
    };

    let sections = collect_listing_sections(
        &[temp_dir.path().display().to_string()],
        &params,
    )
    .unwrap();

    assert_eq!(sections.len(), 1);
    assert!(
        sections[0]
            .entries
            .iter()
            .any(|info| info.display_name.contains("target"))
    );
    assert!(
        !sections.iter().any(
            |section| section.header == Some(target.display().to_string())
        )
    );
}

#[test]
fn test_collect_tree_sections_uses_level_limit() {
    let temp_dir = tempdir().unwrap();
    let child = temp_dir.path().join("child");
    let grandchild = child.join("grandchild");
    fs::create_dir(&child).unwrap();
    fs::create_dir(&grandchild).unwrap();
    fs::write(grandchild.join("deep.txt"), "deep").unwrap();
    let params = Params {
        tree: true,
        long_format: true,
        no_icons: true,
        tree_level: 1,
        ..Params::default()
    };

    let sections = collect_tree_sections(
        &[temp_dir.path().display().to_string()],
        &params,
    )
    .unwrap();

    assert_eq!(sections.len(), 1);
    assert!(
        sections[0]
            .entries
            .iter()
            .any(|entry| entry.info.display_name.contains("child"))
    );
    assert!(
        !sections[0]
            .entries
            .iter()
            .any(|entry| entry.info.display_name.contains("grandchild"))
    );
    assert!(
        !sections[0]
            .entries
            .iter()
            .any(|entry| entry.info.display_name.contains("deep.txt"))
    );
    assert!(
        sections[0]
            .entries
            .iter()
            .any(|entry| entry.name_prefix.is_empty())
    );
}

#[test]
fn test_collect_tree_sections_keeps_prefixes_for_nested_entries() {
    let temp_dir = tempdir().unwrap();
    let child = temp_dir.path().join("child");
    fs::create_dir(&child).unwrap();
    fs::write(child.join("nested.txt"), "nested").unwrap();
    let params = Params {
        tree: true,
        long_format: true,
        no_icons: true,
        tree_level: 2,
        ..Params::default()
    };

    let sections = collect_tree_sections(
        &[temp_dir.path().display().to_string()],
        &params,
    )
    .unwrap();

    let child_entry = sections[0]
        .entries
        .iter()
        .find(|entry| entry.info.display_name.contains("child"))
        .unwrap();
    let nested_entry = sections[0]
        .entries
        .iter()
        .find(|entry| entry.info.display_name.contains("nested.txt"))
        .unwrap();
    assert!(child_entry.name_prefix.is_empty());
    assert!(
        nested_entry.name_prefix.contains("└──")
            || nested_entry.name_prefix.contains("├──")
    );
}

#[test]
fn test_collect_tree_sections_keeps_vertical_prefix_for_non_last_branch() {
    let temp_dir = tempdir().unwrap();
    let first = temp_dir.path().join("a-first");
    let first_branch = first.join("a-branch");
    let first_leaf = first.join("z-leaf");
    let second = temp_dir.path().join("b-second");
    let second_branch = second.join("only-branch");
    fs::create_dir_all(&first_branch).unwrap();
    fs::create_dir_all(&second_branch).unwrap();
    fs::write(&first_leaf, "leaf").unwrap();
    fs::write(first_branch.join("nested.txt"), "nested").unwrap();
    fs::write(second_branch.join("other.txt"), "other").unwrap();
    let params = Params {
        tree: true,
        long_format: true,
        no_icons: true,
        tree_level: 3,
        ..Params::default()
    };

    let sections = collect_tree_sections(
        &[temp_dir.path().display().to_string()],
        &params,
    )
    .unwrap();

    let nested_entry = sections[0]
        .entries
        .iter()
        .find(|entry| entry.info.short_name == "nested.txt")
        .unwrap();
    let other_entry = sections[0]
        .entries
        .iter()
        .find(|entry| entry.info.short_name == "other.txt")
        .unwrap();

    assert!(nested_entry.name_prefix.starts_with("│   "));
    assert!(other_entry.name_prefix.starts_with("    "));
}

#[test]
fn test_collect_tree_sections_handles_file_operand_and_missing_operand() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("shown.txt");
    let missing_path = temp_dir.path().join("missing.txt");
    fs::write(&file_path, "shown").unwrap();
    let params = Params {
        tree: true,
        long_format: true,
        no_icons: true,
        ..Params::default()
    };

    let sections = collect_tree_sections(
        &[
            file_path.display().to_string(),
            missing_path.display().to_string(),
        ],
        &params,
    )
    .unwrap();

    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].header, file_path.display().to_string());
    assert_eq!(sections[0].entries.len(), 1);
    assert_eq!(sections[0].entries[0].info.short_name, "shown.txt");
    assert!(sections[0].entries[0].name_prefix.is_empty());
}

#[test]
fn test_collect_tree_sections_handles_empty_directory() {
    let temp_dir = tempdir().unwrap();
    let params = Params {
        tree: true,
        long_format: true,
        no_icons: true,
        ..Params::default()
    };

    let sections = collect_tree_sections(
        &[temp_dir.path().display().to_string()],
        &params,
    )
    .unwrap();

    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].header, temp_dir.path().display().to_string());
    assert!(sections[0].entries.is_empty());
}

#[test]
fn test_collect_tree_sections_ignores_dot_entries() {
    let temp_dir = tempdir().unwrap();
    fs::write(temp_dir.path().join("shown.txt"), "shown").unwrap();
    let params = Params {
        tree: true,
        long_format: true,
        show_all: true,
        no_icons: true,
        ..Params::default()
    };

    let sections = collect_tree_sections(
        &[temp_dir.path().display().to_string()],
        &params,
    )
    .unwrap();

    assert_eq!(sections.len(), 1);
    assert!(
        sections[0]
            .entries
            .iter()
            .any(|entry| entry.info.display_name.contains("shown.txt"))
    );
    assert!(
        !sections[0]
            .entries
            .iter()
            .any(|entry| entry.info.display_name == "."
                || entry.info.display_name == "..")
    );
}

#[test]
fn test_collect_tree_sections_prunes_noisy_directory_descendants() {
    let temp_dir = tempdir().unwrap();
    let git_dir = temp_dir.path().join(".git");
    fs::create_dir(&git_dir).unwrap();
    fs::write(git_dir.join("config"), "config").unwrap();
    let params = Params {
        tree: true,
        long_format: true,
        show_all: true,
        no_icons: true,
        prune_dirs: vec![String::from(".git")],
        ..Params::default()
    };

    let sections = collect_tree_sections(
        &[temp_dir.path().display().to_string()],
        &params,
    )
    .unwrap();

    assert_eq!(sections.len(), 1);
    assert!(
        sections[0]
            .entries
            .iter()
            .any(|entry| entry.info.display_name.contains(".git"))
    );
    assert!(
        !sections[0]
            .entries
            .iter()
            .any(|entry| entry.info.display_name.contains("config"))
    );
}

#[test]
fn test_collect_tree_sections_prunes_custom_directory_descendants() {
    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().join("target");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("hidden.txt"), "hidden").unwrap();
    let params = Params {
        tree: true,
        long_format: true,
        no_icons: true,
        prune_dirs: vec![String::from("target")],
        ..Params::default()
    };

    let sections = collect_tree_sections(
        &[temp_dir.path().display().to_string()],
        &params,
    )
    .unwrap();

    assert_eq!(sections.len(), 1);
    assert!(
        sections[0]
            .entries
            .iter()
            .any(|entry| entry.info.display_name.contains("target"))
    );
    assert!(
        !sections[0]
            .entries
            .iter()
            .any(|entry| entry.info.display_name.contains("hidden.txt"))
    );
}

#[cfg(unix)]
#[test]
fn test_collect_tree_sections_skips_symlinked_directory_descendants() {
    let temp_dir = tempdir().unwrap();
    let root = temp_dir.path().join("root");
    let real = temp_dir.path().join("real");
    let link = root.join("link");
    fs::create_dir(&root).unwrap();
    fs::create_dir(&real).unwrap();
    fs::write(real.join("deep.txt"), "deep").unwrap();
    std::os::unix::fs::symlink(&real, &link).unwrap();
    let params = Params {
        tree: true,
        long_format: true,
        no_icons: true,
        tree_level: 2,
        ..Params::default()
    };

    let sections =
        collect_tree_sections(&[root.display().to_string()], &params).unwrap();

    assert_eq!(sections.len(), 1);
    assert!(
        sections[0]
            .entries
            .iter()
            .any(|entry| entry.info.short_name.contains("link"))
    );
    assert!(
        !sections[0]
            .entries
            .iter()
            .any(|entry| entry.info.short_name.contains("deep.txt"))
    );
}

#[test]
fn test_color_mode_for_uses_never_when_no_color_is_enabled() {
    let params = Params {
        no_color: true,
        ..Params::default()
    };

    assert_eq!(color_mode_for(&params), ColorMode::Never);
}

#[test]
fn test_color_mode_for_uses_auto_by_default() {
    assert_eq!(color_mode_for(&Params::default()), ColorMode::Auto);
}

#[test]
fn test_long_format_color_level_uses_none_when_color_is_disabled() {
    temp_env::with_var("NO_COLOR", None::<&str>, || {
        let _guard = ColorModeGuard::set(ColorMode::Always);
        let params = Params {
            no_color: true,
            ..Params::default()
        };

        assert_eq!(long_format_color_level(&params), ColorLevel::NoColor);
    });

    temp_env::with_var("NO_COLOR", None::<&str>, || {
        let _guard = ColorModeGuard::set(ColorMode::Never);

        assert_eq!(
            long_format_color_level(&Params::default()),
            ColorLevel::NoColor
        );
    });

    temp_env::with_var("NO_COLOR", Some("1"), || {
        let _guard = ColorModeGuard::set(ColorMode::Always);

        assert_eq!(
            long_format_color_level(&Params::default()),
            ColorLevel::NoColor
        );
    });
}

fn with_terminal_capability_env<T>(
    colorterm: Option<&str>,
    term: Option<&str>,
    test: impl FnOnce() -> T,
) -> T {
    temp_env::with_vars(
        [
            ("COLORTERM", colorterm),
            ("NO_COLOR", None),
            ("TERM", term),
            ("FORCE_COLOR", None),
            ("CLICOLOR", None),
            ("CLICOLOR_FORCE", None),
            ("WT_SESSION", None),
            ("ConEmuANSI", None),
            ("ANSICON", None),
            ("CI", None),
        ],
        test,
    )
}

#[test]
fn test_long_format_color_level_detects_terminal_capability() {
    with_terminal_capability_env(None, Some("dumb"), || {
        let _guard = ColorModeGuard::set(ColorMode::Always);

        assert_eq!(
            long_format_color_level(&Params::default()),
            ColorLevel::NoColor
        );
    });

    with_terminal_capability_env(Some("truecolor"), None, || {
        let _guard = ColorModeGuard::set(ColorMode::Always);

        assert_eq!(
            long_format_color_level(&Params::default()),
            ColorLevel::TrueColor
        );
    });

    with_terminal_capability_env(None, Some("xterm-256color"), || {
        let _guard = ColorModeGuard::set(ColorMode::Always);

        assert_eq!(
            long_format_color_level(&Params::default()),
            ColorLevel::Ansi256
        );
    });

    with_terminal_capability_env(None, Some("xterm"), || {
        let _guard = ColorModeGuard::set(ColorMode::Always);

        assert_eq!(
            long_format_color_level(&Params::default()),
            ColorLevel::Ansi16
        );
    });

    with_terminal_capability_env(None, None, || {
        let _guard = ColorModeGuard::set(ColorMode::Always);

        assert_eq!(
            long_format_color_level(&Params::default()),
            ColorLevel::Ansi16
        );
    });
}
