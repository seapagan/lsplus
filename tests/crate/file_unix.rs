use crate::common_tests::{
    ColorModeGuard, has_ansi, with_color_output_enabled,
};
use crate::platform::{
    LongFormatFileType, get_groupname, get_username, long_format_file_type,
    name_style_for_file_type,
};
use crate::utils::file::{
    collect_file_info, collect_file_names, create_file_info,
    file_type_indicator_suffix_for_type, format_symlink_display_name_with_dim,
};
use crate::utils::icons::Icon;
use crate::{IndicatorStyle, NameStyle, Params};
use colored_text::ColorMode;
use std::ffi::OsString;
use std::fs::{self, File};
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::process::Command;
use strip_ansi_escapes::strip_str;
use tempfile::tempdir;

#[test]
fn test_get_username_and_groupname_fall_back_to_ids() {
    assert!(matches!(get_username(0).as_str(), "root" | "0"));
    assert_eq!(get_username(u32::MAX), u32::MAX.to_string());
    assert!(matches!(get_groupname(0).as_str(), "root" | "0"));
    assert_eq!(get_groupname(u32::MAX), u32::MAX.to_string());
}

#[test]
fn test_collect_file_names_errors_for_broken_directory_symlink() {
    let temp_dir = tempdir().unwrap();
    let broken_dir_link = temp_dir.path().join("broken-dir");
    std::os::unix::fs::symlink("missing-target", &broken_dir_link).unwrap();

    assert!(collect_file_names(&broken_dir_link, &Params::default()).is_err());
}

#[test]
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
            indicator_style: IndicatorStyle::Slash,
            ..Params::default()
        },
    )
    .unwrap();
    assert!(strip_str(&dir_info.display_name).ends_with('/'));

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
fn test_create_file_info_handles_executables_and_large_files() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("large_file");
    let file = File::create(&file_path).unwrap();
    file.set_len(5 * 1024 * 1024 * 1024).unwrap();
    fs::set_permissions(&file_path, fs::Permissions::from_mode(0o755))
        .unwrap();

    with_color_output_enabled(|| {
        let info = create_file_info(
            &file_path,
            &Params {
                human_readable: true,
                ..Params::default()
            },
        )
        .unwrap();

        assert_eq!(info.size, 5 * 1024 * 1024 * 1024);
        assert!(info.display_name.contains("\u{1b}[1;32m"));
    });
}

#[test]
fn test_create_file_info_returns_plain_names_when_color_is_disabled() {
    let _guard = ColorModeGuard::set(ColorMode::Never);
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("plain-file");
    let dir_path = temp_dir.path().join("plain-dir");
    let link_path = temp_dir.path().join("plain-link");

    fs::write(&file_path, "file").unwrap();
    fs::create_dir(&dir_path).unwrap();
    std::os::unix::fs::symlink(&file_path, &link_path).unwrap();

    let file_info = create_file_info(&file_path, &Params::default()).unwrap();
    let dir_info = create_file_info(&dir_path, &Params::default()).unwrap();
    let link_info = create_file_info(
        &link_path,
        &Params {
            long_format: true,
            ..Params::default()
        },
    )
    .unwrap();

    assert!(!has_ansi(&file_info.display_name));
    assert!(!has_ansi(&dir_info.display_name));
    assert!(!has_ansi(&link_info.display_name));
    assert!(link_info.display_name.contains("->"));
}

#[test]
fn test_create_file_info_marks_fifo_as_pipe_type() {
    let temp_dir = tempdir().unwrap();
    let fifo_path = temp_dir.path().join("pipe");
    let status = Command::new("mkfifo").arg(&fifo_path).status().unwrap();
    assert!(status.success());

    let info = create_file_info(&fifo_path, &Params::default()).unwrap();

    assert_eq!(info.file_type, "p");
}

#[test]
fn test_create_file_info_classify_prefers_fifo_and_socket_indicators() {
    let temp_dir = tempdir().unwrap();
    let fifo_path = temp_dir.path().join("pipe");
    let socket_path = temp_dir.path().join("socket");
    let status = Command::new("mkfifo").arg(&fifo_path).status().unwrap();
    assert!(status.success());

    let _listener = UnixListener::bind(&socket_path).unwrap();

    fs::set_permissions(&fifo_path, fs::Permissions::from_mode(0o755))
        .unwrap();
    fs::set_permissions(&socket_path, fs::Permissions::from_mode(0o755))
        .unwrap();

    let fifo_metadata = fs::symlink_metadata(&fifo_path).unwrap();
    let socket_metadata = fs::symlink_metadata(&socket_path).unwrap();
    assert_ne!(fifo_metadata.permissions().mode() & 0o111, 0);
    assert_ne!(socket_metadata.permissions().mode() & 0o111, 0);

    let params = Params {
        indicator_style: IndicatorStyle::Classify,
        ..Params::default()
    };
    let fifo_info = create_file_info(&fifo_path, &params).unwrap();
    let socket_info = create_file_info(&socket_path, &params).unwrap();

    assert_eq!(fifo_info.file_type, "p");
    assert_eq!(socket_info.file_type, "s");
    assert_eq!(fifo_info.name_style, NameStyle::Fifo);
    assert_eq!(socket_info.name_style, NameStyle::Socket);
    assert_eq!(fifo_info.item_icon, Some(Icon::PipeFile));
    assert_eq!(socket_info.item_icon, Some(Icon::SocketFile));
    assert!(strip_str(&fifo_info.display_name).ends_with('|'));
    assert!(!strip_str(&fifo_info.display_name).ends_with('*'));
    assert!(strip_str(&socket_info.display_name).ends_with('='));
    assert!(!strip_str(&socket_info.display_name).ends_with('*'));
}

#[test]
fn test_file_type_indicator_suffix_for_unix_special_types() {
    let cases = [
        (LongFormatFileType::Directory, false, true, "/"),
        (LongFormatFileType::Symlink, false, true, "@"),
        (LongFormatFileType::Fifo, true, true, "|"),
        (LongFormatFileType::Socket, true, true, "="),
        (LongFormatFileType::Regular, true, true, "*"),
        (LongFormatFileType::Regular, false, true, ""),
        (LongFormatFileType::CharDevice, true, true, ""),
        (LongFormatFileType::BlockDevice, true, true, ""),
        (LongFormatFileType::Unknown, true, true, ""),
    ];

    for (file_type, classify_executables, executable, expected) in cases {
        assert_eq!(
            file_type_indicator_suffix_for_type(
                file_type,
                classify_executables,
                executable
            ),
            expected
        );
    }
}

#[test]
fn test_create_file_info_colors_special_file_names() {
    with_color_output_enabled(|| {
        let temp_dir = tempdir().unwrap();
        let fifo_path = temp_dir.path().join("pipe");
        let socket_path = temp_dir.path().join("socket");
        let status = Command::new("mkfifo").arg(&fifo_path).status().unwrap();
        assert!(status.success());

        let _listener = UnixListener::bind(&socket_path).unwrap();

        fs::set_permissions(&fifo_path, fs::Permissions::from_mode(0o755))
            .unwrap();
        fs::set_permissions(&socket_path, fs::Permissions::from_mode(0o755))
            .unwrap();

        let fifo_info =
            create_file_info(&fifo_path, &Params::default()).unwrap();
        let socket_info =
            create_file_info(&socket_path, &Params::default()).unwrap();

        assert!(fifo_info.display_name.contains("\u{1b}[33m"));
        assert!(!fifo_info.display_name.contains("\u{1b}[1;32m"));
        assert!(socket_info.display_name.contains("\u{1b}[1;35m"));
        assert!(!socket_info.display_name.contains("\u{1b}[1;32m"));
    });
}

#[test]
fn test_create_file_info_colors_char_device_names() {
    let metadata = fs::symlink_metadata("/dev/null").unwrap();
    if !metadata.file_type().is_char_device() {
        return;
    }

    with_color_output_enabled(|| {
        let info =
            create_file_info(Path::new("/dev/null"), &Params::default())
                .unwrap();

        assert_eq!(info.file_type, "c");
        assert_eq!(info.name_style, NameStyle::CharDevice);
        assert_eq!(info.item_icon, Some(Icon::CharDeviceFile));
        assert!(info.display_name.contains("\u{1b}[1;33m"));
    });
}

#[test]
fn test_long_format_file_type_chars_for_unix_special_types() {
    let cases = [
        (LongFormatFileType::Fifo, 'p'),
        (LongFormatFileType::Socket, 's'),
        (LongFormatFileType::CharDevice, 'c'),
        (LongFormatFileType::BlockDevice, 'b'),
        (LongFormatFileType::Unknown, '?'),
    ];

    for (file_type, expected) in cases {
        assert_eq!(file_type.as_char(), expected);
    }
}

#[test]
fn test_long_format_file_type_maps_unix_file_type_bits() {
    let cases = [
        (0o040755, LongFormatFileType::Directory),
        (0o100644, LongFormatFileType::Regular),
        (0o120777, LongFormatFileType::Symlink),
        (0o140777, LongFormatFileType::Socket),
        (0o010644, LongFormatFileType::Fifo),
        (0o020644, LongFormatFileType::CharDevice),
        (0o060644, LongFormatFileType::BlockDevice),
        (0, LongFormatFileType::Unknown),
    ];

    for (mode, expected) in cases {
        assert_eq!(long_format_file_type(mode), expected);
    }
}

#[test]
fn test_name_style_for_unix_special_file_types() {
    let cases = [
        (LongFormatFileType::Socket, NameStyle::Socket),
        (LongFormatFileType::Fifo, NameStyle::Fifo),
        (LongFormatFileType::CharDevice, NameStyle::CharDevice),
        (LongFormatFileType::BlockDevice, NameStyle::BlockDevice),
        (LongFormatFileType::Regular, NameStyle::Executable),
        (LongFormatFileType::Unknown, NameStyle::Plain),
    ];

    for (file_type, expected) in cases {
        assert_eq!(name_style_for_file_type(file_type, true), expected);
    }
}

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
fn test_create_file_info_omits_symlink_at_in_long_mode() {
    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().join("target.txt");
    let link = temp_dir.path().join("link");

    fs::write(&target, "target").unwrap();
    std::os::unix::fs::symlink(&target, &link).unwrap();

    let info = create_file_info(
        &link,
        &Params {
            long_format: true,
            indicator_style: IndicatorStyle::FileType,
            ..Params::default()
        },
    )
    .unwrap();

    assert!(strip_str(&info.display_name).contains("link -> "));
    assert!(!strip_str(&info.display_name).contains("link@ -> "));
}

#[test]
fn test_format_symlink_display_name_colors_long_format_targets_by_type() {
    with_color_output_enabled(|| {
        let temp_dir = tempdir().unwrap();
        let dir_target = temp_dir.path().join("dir-target");
        let file_target = temp_dir.path().join("file-target.txt");
        let symlink_target = temp_dir.path().join("symlink-target");
        let exec_target = temp_dir.path().join("exec-target.sh");

        fs::create_dir(&dir_target).unwrap();
        fs::write(&file_target, "file").unwrap();
        fs::write(&exec_target, "#!/bin/sh\nexit 0\n").unwrap();
        fs::set_permissions(&exec_target, fs::Permissions::from_mode(0o755))
            .unwrap();
        std::os::unix::fs::symlink(&file_target, &symlink_target).unwrap();

        let params = Params {
            long_format: true,
            ..Params::default()
        };

        let dir_display = format_symlink_display_name_with_dim(
            "dir-link",
            &temp_dir.path().join("dir-link"),
            Ok(PathBuf::from("dir-target")),
            &params,
            NameStyle::Symlink,
            false,
        );
        assert!(dir_display.contains("-> \u{1b}[34m"));

        let file_display = format_symlink_display_name_with_dim(
            "file-link",
            &temp_dir.path().join("file-link"),
            Ok(PathBuf::from("file-target.txt")),
            &params,
            NameStyle::Symlink,
            false,
        );
        assert!(!file_display.contains("-> \u{1b}["));

        let symlink_display = format_symlink_display_name_with_dim(
            "symlink-link",
            &temp_dir.path().join("symlink-link"),
            Ok(PathBuf::from("symlink-target")),
            &params,
            NameStyle::Symlink,
            false,
        );
        assert!(symlink_display.contains("-> \u{1b}[36m"));

        let exec_display = format_symlink_display_name_with_dim(
            "exec-link",
            &temp_dir.path().join("exec-link"),
            Ok(PathBuf::from("exec-target.sh")),
            &params,
            NameStyle::Symlink,
            false,
        );
        assert!(exec_display.contains("-> \u{1b}[1;32m"));
    });
}

#[test]
fn test_gitignored_entries_remain_plain_when_color_is_disabled() {
    let _guard = ColorModeGuard::set(ColorMode::Never);
    let temp_dir = tempdir().unwrap();
    fs::create_dir(temp_dir.path().join(".git")).unwrap();
    fs::write(temp_dir.path().join(".gitignore"), "*.log\n").unwrap();
    let ignored_path = temp_dir.path().join("ignored.log");
    fs::write(&ignored_path, "ignored").unwrap();

    let info = create_file_info(
        &ignored_path,
        &Params {
            gitignore: true,
            ..Params::default()
        },
    )
    .unwrap();

    assert_eq!(info.display_name, "ignored.log");
}

#[test]
fn test_gitignored_entries_are_dimmed_when_color_is_enabled() {
    with_color_output_enabled(|| {
        let temp_dir = tempdir().unwrap();
        fs::create_dir(temp_dir.path().join(".git")).unwrap();
        fs::write(temp_dir.path().join(".gitignore"), "*.sh\n").unwrap();
        let ignored_path = temp_dir.path().join("ignored.sh");
        fs::write(&ignored_path, "#!/bin/sh\n").unwrap();
        fs::set_permissions(&ignored_path, fs::Permissions::from_mode(0o755))
            .unwrap();

        let info = create_file_info(
            &ignored_path,
            &Params {
                gitignore: true,
                ..Params::default()
            },
        )
        .unwrap();

        assert!(has_ansi(&info.display_name));
        assert!(info.display_name.contains("ignored.sh"));
        assert!(
            info.display_name.contains("\u{1b}[1;2;32m")
                || info.display_name.contains("\u{1b}[2;1;32m")
        );
    });
}
