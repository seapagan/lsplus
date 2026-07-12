use crate::common_tests::with_color_output_enabled;
#[cfg(unix)]
use crate::platform::{EntryClassification, LongFormatFileType};
#[cfg(unix)]
use crate::utils::file::DirectoryEntryData;
use crate::utils::file::{
    append_file_info_for_names, check_display_name, collect_file_info,
    collect_file_names, collect_visible_file_names, create_file_info,
    format_path_error, format_symlink_display_name_with_dim,
    preserve_synthetic_dot_name, sanitize_for_terminal,
};
use crate::{FileInfo, IndicatorStyle, NameStyle, Params};
#[cfg(unix)]
use std::ffi::OsString;
use std::fs;
#[cfg(unix)]
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tempfile::tempdir;

const BLUE_DOT: &str = "\u{1b}[34m.\u{1b}[0m";
const BLUE_DOTDOT: &str = "\u{1b}[34m..\u{1b}[0m";

fn basic_info(display_name: &str, full_path: PathBuf) -> FileInfo {
    FileInfo {
        file_type: String::from("directory"),
        mode: String::from("drwxr-xr-x"),
        mode_bits: 0o755,
        nlink: 1,
        user: String::from("user"),
        group: String::from("group"),
        size: 0,
        mtime: SystemTime::now(),
        item_icon: None,
        short_name: display_name.to_string(),
        display_name: display_name.to_string(),
        name_style: NameStyle::Plain,
        dimmed: false,
        full_path,
    }
}

#[test]
fn test_check_display_name_handles_regular_and_special_entries() {
    let plain = basic_info("test.txt", PathBuf::from("test.txt"));
    assert_eq!(check_display_name(&plain), "test.txt");

    with_color_output_enabled(|| {
        let dot = basic_info(".", PathBuf::from("/tmp/."));
        assert_eq!(check_display_name(&dot), BLUE_DOT);

        let dotdot = basic_info("..", PathBuf::from("/tmp/.."));
        assert_eq!(check_display_name(&dotdot), BLUE_DOTDOT);
    });
}

#[test]
fn test_preserve_synthetic_dot_name_updates_only_dot_entries() {
    let mut dot = basic_info("normalized", PathBuf::from("."));
    preserve_synthetic_dot_name(&mut dot, ".");
    assert_eq!(dot.short_name, ".");
    assert_eq!(dot.display_name, ".");

    let mut regular = basic_info("regular", PathBuf::from("regular"));
    preserve_synthetic_dot_name(&mut regular, "other");
    assert_eq!(regular.short_name, "regular");
    assert_eq!(regular.display_name, "regular");
}

#[test]
#[cfg(unix)]
fn test_collect_file_info_respects_visibility_and_directory_order() {
    let temp_dir = tempdir().unwrap();
    fs::write(temp_dir.path().join("file1.txt"), "one").unwrap();
    fs::write(temp_dir.path().join("file2.txt"), "two").unwrap();
    fs::write(temp_dir.path().join(".hidden"), "hidden").unwrap();
    fs::create_dir(temp_dir.path().join("dir1")).unwrap();

    let info = collect_file_info(temp_dir.path(), &Params::default()).unwrap();
    assert_eq!(info.len(), 3);

    let info = collect_file_info(
        temp_dir.path(),
        &Params {
            show_all: true,
            ..Params::default()
        },
    )
    .unwrap();
    assert_eq!(info.len(), 6);

    let info = collect_file_info(
        temp_dir.path(),
        &Params {
            dirs_first: true,
            ..Params::default()
        },
    )
    .unwrap();
    let first_file_idx = info.iter().position(|f| f.file_type != "d").unwrap();
    assert!(info[..first_file_idx].iter().all(|f| f.file_type == "d"));
    assert!(info[first_file_idx..].iter().all(|f| f.file_type != "d"));
}

#[test]
fn test_collect_file_info_handles_direct_file_arguments() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "test content").unwrap();

    let info = collect_file_info(&file_path, &Params::default()).unwrap();

    assert_eq!(info.len(), 1);
    assert_eq!(info[0].size, 12);
    assert!(info[0].display_name.contains("test.txt"));
}

#[test]
fn test_file_helpers_return_errors_for_missing_paths() {
    let temp_dir = tempdir().unwrap();
    let missing = temp_dir.path().join("missing.txt");

    assert!(collect_file_info(&missing, &Params::default()).is_err());
    assert!(create_file_info(&missing, &Params::default()).is_err());
    assert!(collect_file_names(&missing, &Params::default()).is_err());
}

#[test]
#[cfg(unix)]
fn test_collect_file_names_handles_visibility_flags_and_regular_files() {
    let temp_dir = tempdir().unwrap();
    let hidden_path = temp_dir.path().join(".hidden_file");
    let visible_path = temp_dir.path().join("visible_file");
    let subdir_path = temp_dir.path().join("subdir");

    fs::create_dir(&subdir_path).unwrap();
    File::create(&hidden_path).unwrap();
    File::create(&visible_path).unwrap();

    let files =
        collect_file_names(temp_dir.path(), &Params::default()).unwrap();
    assert!(!files.contains(&".hidden_file".to_string()));
    assert!(files.contains(&"visible_file".to_string()));

    let files = collect_file_names(
        temp_dir.path(),
        &Params {
            show_all: true,
            ..Params::default()
        },
    )
    .unwrap();
    assert!(files.contains(&".".to_string()));
    assert!(files.contains(&"..".to_string()));
    assert!(files.contains(&".hidden_file".to_string()));

    let files = collect_file_names(
        temp_dir.path(),
        &Params {
            almost_all: true,
            ..Params::default()
        },
    )
    .unwrap();
    assert!(!files.contains(&".".to_string()));
    assert!(!files.contains(&"..".to_string()));
    assert!(files.contains(&".hidden_file".to_string()));

    let files = collect_file_names(
        temp_dir.path(),
        &Params {
            dirs_first: true,
            ..Params::default()
        },
    )
    .unwrap();
    let subdir_idx = files.iter().position(|x| x == "subdir").unwrap();
    let file_idx = files.iter().position(|x| x == "visible_file").unwrap();
    assert!(subdir_idx < file_idx);

    assert_eq!(
        collect_file_names(&visible_path, &Params::default()).unwrap(),
        vec!["visible_file"]
    );
}

#[test]
#[cfg(unix)]
fn test_collect_visible_file_names_handles_iterator_and_file_type_errors() {
    let params = Params {
        show_all: true,
        dirs_first: true,
        ..Params::default()
    };
    let entries = vec![
        Err(io::Error::other("iterator error")),
        Ok(DirectoryEntryData {
            file_name: OsString::from("alpha.txt"),
            path: PathBuf::from("/tmp/alpha.txt"),
            classification_result: Ok(EntryClassification {
                file_type: LongFormatFileType::Regular,
                hidden: false,
                display_as_directory: false,
                group_with_directories: false,
                may_recurse: false,
                may_render_link_target: false,
            }),
        }),
        Ok(DirectoryEntryData {
            file_name: OsString::from("broken"),
            path: PathBuf::from("/tmp/broken"),
            classification_result: Err(io::Error::other("type error")),
        }),
        Ok(DirectoryEntryData {
            file_name: OsString::from("dir"),
            path: PathBuf::from("/tmp/dir"),
            classification_result: Ok(EntryClassification {
                file_type: LongFormatFileType::Directory,
                hidden: false,
                display_as_directory: true,
                group_with_directories: true,
                may_recurse: true,
                may_render_link_target: false,
            }),
        }),
    ];

    let names =
        collect_visible_file_names(Path::new("/tmp"), entries, &params);

    assert_eq!(names[0], ".");
    assert_eq!(names[1], "..");
    assert!(names.iter().any(|name| name == "dir"));
    assert!(names.iter().any(|name| name == "alpha.txt"));
    assert!(names.iter().any(|name| name == "broken"));
}

#[test]
fn test_append_file_info_for_names_skips_missing_entries() {
    let temp_dir = tempdir().unwrap();
    let existing = temp_dir.path().join("existing.txt");
    fs::write(&existing, "existing").unwrap();

    let file_names =
        vec![String::from("existing.txt"), String::from("missing.txt")];
    let mut file_info = Vec::new();
    let mut gitignore_cache =
        crate::utils::gitignore::GitignoreCache::default();

    append_file_info_for_names(
        &mut file_info,
        temp_dir.path(),
        &file_names,
        &Params::default(),
        &mut gitignore_cache,
    );

    assert_eq!(file_info.len(), 1);
    assert!(file_info[0].display_name.contains("existing.txt"));
}

#[test]
fn test_append_file_info_for_names_handles_empty_input() {
    let mut file_info = Vec::new();
    let mut gitignore_cache =
        crate::utils::gitignore::GitignoreCache::default();

    append_file_info_for_names(
        &mut file_info,
        Path::new("/tmp"),
        &[],
        &Params::default(),
        &mut gitignore_cache,
    );

    assert!(file_info.is_empty());
}

#[test]
fn test_format_symlink_display_name_handles_unreadable_targets() {
    let params = Params {
        long_format: true,
        indicator_style: IndicatorStyle::FileType,
        ..Params::default()
    };
    let unreadable = format_symlink_display_name_with_dim(
        "broken-link",
        Path::new("/tmp/broken-link"),
        Err(io::Error::other("boom")),
        &params,
        NameStyle::Symlink,
        false,
    );
    assert!(unreadable.contains("[Target Unavailable]"));

    let short = format_symlink_display_name_with_dim(
        "broken-link@",
        Path::new("/tmp/broken-link"),
        Err(io::Error::other("boom")),
        &Params {
            indicator_style: IndicatorStyle::FileType,
            ..Params::default()
        },
        NameStyle::Symlink,
        false,
    );
    assert!(short.contains('@'));
}

#[test]
fn test_sanitize_for_terminal_and_format_path_error_escape_controls() {
    let sanitized = sanitize_for_terminal("bad\n\r\t\u{1b}name");
    assert_eq!(sanitized, "bad\\n\\r\\t\\x1bname");
    assert_eq!(sanitize_for_terminal(""), "");

    let err = io::Error::other("boom");
    let formatted = format_path_error(Path::new("bad\nname"), &err);
    assert!(formatted.contains("bad\\nname"));
    assert!(formatted.contains("boom"));
}

#[test]
fn test_collect_visible_file_names_handles_empty_entries() {
    let names = collect_visible_file_names(
        Path::new("/tmp"),
        Vec::new(),
        &Params::default(),
    );

    assert!(names.is_empty());
}

#[test]
fn test_format_symlink_display_name_short_format_omits_marker_without_indicator()
 {
    let short = format_symlink_display_name_with_dim(
        "link",
        Path::new("/tmp/link"),
        Ok(PathBuf::from("target")),
        &Params::default(),
        NameStyle::Symlink,
        false,
    );

    assert!(short.contains("link"));
    assert!(!short.contains('*'));
    assert!(!short.contains('@'));
}

#[test]
fn test_format_symlink_display_name_short_format_uses_at_for_file_type() {
    let short = format_symlink_display_name_with_dim(
        "link@",
        Path::new("/tmp/link"),
        Ok(PathBuf::from("target")),
        &Params {
            indicator_style: IndicatorStyle::FileType,
            ..Params::default()
        },
        NameStyle::Symlink,
        false,
    );

    assert!(short.contains("link"));
    assert!(short.contains('@'));
    assert!(!short.contains('*'));
}

#[test]
fn test_format_symlink_display_name_unreadable_short_format_omits_marker_without_indicator()
 {
    let short = format_symlink_display_name_with_dim(
        "link",
        Path::new("/tmp/link"),
        Err(io::Error::other("boom")),
        &Params::default(),
        NameStyle::Symlink,
        false,
    );

    assert!(short.contains("link"));
    assert!(!short.contains('*'));
    assert!(!short.contains('@'));
}
