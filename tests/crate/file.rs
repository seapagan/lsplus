use crate::utils::file::{
    DirectoryEntryData, append_file_info_for_names, check_display_name,
    collect_file_info, collect_file_names, collect_visible_file_names,
    create_file_info, format_path_error, format_symlink_display_name,
    get_groupname, get_username, sanitize_for_terminal,
};
use crate::{FileInfo, Params};
use inline_colorization::{color_blue, color_green};
use std::ffi::OsString;
use std::fs::{self, File};
use std::io;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;
use strip_ansi_escapes::strip_str;
use tempfile::tempdir;

#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;

fn basic_info(display_name: &str, full_path: PathBuf) -> FileInfo {
    FileInfo {
        file_type: String::from("directory"),
        mode: String::from("drwxr-xr-x"),
        nlink: 1,
        user: String::from("user"),
        group: String::from("group"),
        size: 0,
        mtime: SystemTime::now(),
        item_icon: None,
        display_name: display_name.to_string(),
        full_path,
    }
}

#[test]
fn test_check_display_name_handles_regular_and_special_entries() {
    let plain = basic_info("test.txt", PathBuf::from("test.txt"));
    assert_eq!(check_display_name(&plain), "test.txt");

    let dot = basic_info(".", PathBuf::from("/tmp/."));
    assert_eq!(check_display_name(&dot), format!("{color_blue}."));

    let dotdot = basic_info("..", PathBuf::from("/tmp/.."));
    assert_eq!(check_display_name(&dotdot), format!("{color_blue}.."));
}

#[test]
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
fn test_file_helpers_return_errors_for_missing_paths_and_broken_symlinks() {
    let temp_dir = tempdir().unwrap();
    let missing = temp_dir.path().join("missing.txt");
    let broken_dir_link = temp_dir.path().join("broken-dir");

    assert!(collect_file_info(&missing, &Params::default()).is_err());
    assert!(create_file_info(&missing, &Params::default()).is_err());
    assert!(collect_file_names(&missing, &Params::default()).is_err());

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("missing-target", &broken_dir_link)
            .unwrap();
        assert!(
            collect_file_names(&broken_dir_link, &Params::default()).is_err()
        );
    }
}

#[test]
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
            is_dir: Ok(false),
        }),
        Ok(DirectoryEntryData {
            file_name: OsString::from("broken"),
            path: PathBuf::from("/tmp/broken"),
            is_dir: Err(io::Error::other("type error")),
        }),
        Ok(DirectoryEntryData {
            file_name: OsString::from("dir"),
            path: PathBuf::from("/tmp/dir"),
            is_dir: Ok(true),
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
#[cfg(unix)]
fn test_create_file_info_handles_regular_files_symlinks_and_special_cases() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test_file");
    let dir_path = temp_dir.path().join("test_dir");
    let valid_symlink = temp_dir.path().join("valid_symlink");
    let broken_symlink = temp_dir.path().join("broken_symlink");

    fs::create_dir(&dir_path).unwrap();
    File::create(&file_path).unwrap();
    std::os::unix::fs::symlink(&file_path, &valid_symlink).unwrap();
    std::os::unix::fs::symlink("missing-target", &broken_symlink).unwrap();

    let file_info = create_file_info(&file_path, &Params::default()).unwrap();
    assert!(file_info.display_name.contains("test_file"));

    let dir_info = create_file_info(
        &dir_path,
        &Params {
            append_slash: true,
            ..Params::default()
        },
    )
    .unwrap();
    assert!(dir_info.display_name.ends_with('/'));

    let link_info = create_file_info(
        &valid_symlink,
        &Params {
            long_format: true,
            ..Params::default()
        },
    )
    .unwrap();
    assert!(link_info.display_name.contains("->"));

    let broken_info = create_file_info(
        &broken_symlink,
        &Params {
            long_format: true,
            ..Params::default()
        },
    )
    .unwrap();
    assert!(broken_info.display_name.contains("[Broken Link]"));
}

#[test]
#[cfg(unix)]
fn test_create_file_info_handles_executables_and_large_files() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("large_file");
    let file = File::create(&file_path).unwrap();
    file.set_len(5 * 1024 * 1024 * 1024).unwrap();
    fs::set_permissions(&file_path, fs::Permissions::from_mode(0o755))
        .unwrap();

    let info = create_file_info(
        &file_path,
        &Params {
            human_readable: true,
            ..Params::default()
        },
    )
    .unwrap();

    assert_eq!(info.size, 5 * 1024 * 1024 * 1024);
    assert!(info.display_name.contains(color_green));
}

#[cfg(unix)]
#[test]
fn test_create_file_info_marks_fifo_as_unknown_type() {
    let temp_dir = tempdir().unwrap();
    let fifo_path = temp_dir.path().join("pipe");
    let status = Command::new("mkfifo").arg(&fifo_path).status().unwrap();
    assert!(status.success());

    let info = create_file_info(&fifo_path, &Params::default()).unwrap();

    assert_eq!(info.file_type, "?");
}

#[cfg(unix)]
#[test]
fn test_create_file_info_handles_non_utf8_names_and_control_characters() {
    let temp_dir = tempdir().unwrap();
    let invalid_name = OsString::from_vec(vec![b'f', b'o', 0xff, b'o']);
    let invalid_path = temp_dir.path().join(&invalid_name);
    File::create(&invalid_path).unwrap();

    let files =
        collect_file_names(temp_dir.path(), &Params::default()).unwrap();
    assert!(files.iter().any(|name| name.contains('\u{fffd}')));

    let unsafe_path = temp_dir.path().join("bad\n\r\t\u{202e}name");
    File::create(&unsafe_path).unwrap();
    let info = create_file_info(&unsafe_path, &Params::default()).unwrap();
    let cleaned_name = strip_str(&info.display_name);

    assert!(cleaned_name.contains("\\n"));
    assert!(cleaned_name.contains("\\r"));
    assert!(cleaned_name.contains("\\t"));
    assert!(cleaned_name.contains('\u{202e}'));
}

#[test]
#[cfg(unix)]
fn test_symlink_directory_and_circular_links_are_handled() {
    let temp_dir = tempdir().unwrap();
    let target_dir = temp_dir.path().join("target_dir");
    let symlink_dir = temp_dir.path().join("symlink_dir");
    let file_in_dir = target_dir.join("test_file.txt");
    let link1 = temp_dir.path().join("link1");
    let link2 = temp_dir.path().join("link2");

    fs::create_dir(&target_dir).unwrap();
    fs::write(&file_in_dir, "test content").unwrap();
    std::os::unix::fs::symlink(&target_dir, &symlink_dir).unwrap();
    std::os::unix::fs::symlink(&link2, &link1).unwrap();
    std::os::unix::fs::symlink(&link1, &link2).unwrap();

    let files = collect_file_names(&symlink_dir, &Params::default()).unwrap();
    assert_eq!(files, vec![String::from("test_file.txt")]);

    let info = collect_file_info(&symlink_dir, &Params::default()).unwrap();
    assert_eq!(info.len(), 1);

    let circular = create_file_info(
        &link1,
        &Params {
            long_format: true,
            ..Params::default()
        },
    )
    .unwrap();
    assert_eq!(circular.file_type, "l");
    assert!(circular.display_name.contains("->"));
}

#[test]
fn test_get_username_and_groupname_fall_back_to_ids() {
    assert!(matches!(get_username(0).as_str(), "root" | "0"));
    assert_eq!(get_username(u32::MAX), u32::MAX.to_string());
    assert!(matches!(get_groupname(0).as_str(), "root" | "0"));
    assert_eq!(get_groupname(u32::MAX), u32::MAX.to_string());
}

#[test]
fn test_format_symlink_display_name_handles_unreadable_targets() {
    let params = Params {
        long_format: true,
        append_slash: true,
        ..Params::default()
    };
    let unreadable = format_symlink_display_name(
        "broken-link",
        Path::new("/tmp/broken-link"),
        Err(io::Error::other("boom")),
        &params,
    );
    assert!(unreadable.contains("(unreadable)"));

    let short = format_symlink_display_name(
        "broken-link",
        Path::new("/tmp/broken-link"),
        Err(io::Error::other("boom")),
        &Params {
            append_slash: true,
            ..Params::default()
        },
    );
    assert!(short.contains('*'));
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
fn test_format_symlink_display_name_short_format_omits_marker_without_append_slash()
 {
    let short = format_symlink_display_name(
        "link",
        Path::new("/tmp/link"),
        Ok(PathBuf::from("target")),
        &Params::default(),
    );

    assert!(short.contains("link"));
    assert!(!short.contains('*'));
}

#[test]
fn test_format_symlink_display_name_short_format_marks_append_slash() {
    let short = format_symlink_display_name(
        "link",
        Path::new("/tmp/link"),
        Ok(PathBuf::from("target")),
        &Params {
            append_slash: true,
            ..Params::default()
        },
    );

    assert!(short.contains("link"));
    assert!(short.contains('*'));
}

#[test]
fn test_format_symlink_display_name_unreadable_short_format_omits_marker_without_append_slash()
 {
    let short = format_symlink_display_name(
        "link",
        Path::new("/tmp/link"),
        Err(io::Error::other("boom")),
        &Params::default(),
    );

    assert!(short.contains("link"));
    assert!(!short.contains('*'));
}
