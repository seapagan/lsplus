use crate::common_tests::with_color_output_enabled;
use crate::platform::{
    EntryClassification, LongColumn, LongFormatFileType,
    LongFormatLayoutOptions, attribute_text, classify_entry,
    compare_entry_names, compare_result_ordering, default_config_path,
    entry_name_is_hidden, extended_find_path,
    extended_find_path_with_current_dir, long_format_layout,
    name_style_for_file_type, non_reparse_file_type, normalize_path,
    parse_pathext, reparse_file_type, validate_params,
};
use crate::structs::{AttributeDisplay, PermissionDisplay};
use crate::utils::file::{
    DirectoryEntryData, collect_visible_file_names, colorize_name,
    file_type_indicator_suffix_for_type, format_symlink_display_name_with_dim,
    slash_indicator_suffix,
};
use crate::{Params, structs::NameStyle};
use std::cmp::Ordering;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

#[test]
fn test_windows_layout_uses_native_columns() {
    let layout = long_format_layout(&LongFormatLayoutOptions {
        permission_display: PermissionDisplay::Symbolic,
        include_size_unit: true,
        include_icon: true,
    });

    assert_eq!(
        layout.columns,
        vec![
            LongColumn::Type,
            LongColumn::Attributes,
            LongColumn::Size,
            LongColumn::Unit,
            LongColumn::Date,
            LongColumn::Icon,
            LongColumn::Name,
        ]
    );
}

#[test]
fn test_windows_layout_omits_optional_columns() {
    let layout = long_format_layout(&LongFormatLayoutOptions {
        permission_display: PermissionDisplay::None,
        include_size_unit: false,
        include_icon: false,
    });

    assert_eq!(
        layout.columns,
        vec![
            LongColumn::Type,
            LongColumn::Size,
            LongColumn::Date,
            LongColumn::Name
        ]
    );
}

#[test]
fn test_windows_shared_file_type_mappings_cover_every_variant() {
    let cases = [
        (LongFormatFileType::Directory, 'd', NameStyle::Directory),
        (LongFormatFileType::Regular, '-', NameStyle::Executable),
        (LongFormatFileType::Symlink, 'l', NameStyle::Symlink),
        (LongFormatFileType::SymlinkFile, 'l', NameStyle::Symlink),
        (
            LongFormatFileType::SymlinkDirectory,
            'L',
            NameStyle::Symlink,
        ),
        (LongFormatFileType::Junction, 'j', NameStyle::Junction),
        (LongFormatFileType::ReparsePoint, 'r', NameStyle::Plain),
        (LongFormatFileType::Socket, 's', NameStyle::Socket),
        (LongFormatFileType::Fifo, 'p', NameStyle::Fifo),
        (LongFormatFileType::CharDevice, 'c', NameStyle::CharDevice),
        (LongFormatFileType::BlockDevice, 'b', NameStyle::BlockDevice),
        (LongFormatFileType::Unknown, '?', NameStyle::Plain),
    ];

    for (file_type, marker, style) in cases {
        assert_eq!(file_type.as_char(), marker);
        assert_eq!(name_style_for_file_type(file_type, true), style);
    }
    assert_eq!(
        name_style_for_file_type(LongFormatFileType::Regular, false),
        NameStyle::Plain
    );
}

#[test]
fn test_windows_shared_indicator_suffixes_cover_special_types() {
    for (file_type, expected) in [
        (LongFormatFileType::Directory, "/"),
        (LongFormatFileType::Symlink, "@"),
        (LongFormatFileType::SymlinkFile, "@"),
        (LongFormatFileType::SymlinkDirectory, "@"),
        (LongFormatFileType::Junction, "@"),
        (LongFormatFileType::Fifo, "|"),
        (LongFormatFileType::Socket, "="),
        (LongFormatFileType::CharDevice, ""),
        (LongFormatFileType::BlockDevice, ""),
        (LongFormatFileType::ReparsePoint, ""),
        (LongFormatFileType::Unknown, ""),
    ] {
        assert_eq!(
            file_type_indicator_suffix_for_type(file_type, true, false),
            expected
        );
    }
    assert_eq!(
        file_type_indicator_suffix_for_type(
            LongFormatFileType::Regular,
            true,
            true,
        ),
        "*"
    );
}

#[test]
fn test_windows_name_fallback_is_never_hidden() {
    assert!(!entry_name_is_hidden(OsStr::new(".dotfile")));
}

#[test]
fn test_windows_colorizes_every_resolved_name_style() {
    with_color_output_enabled(|| {
        for (style, expected) in [
            (NameStyle::Plain, "name"),
            (NameStyle::Directory, "\u{1b}[34mname\u{1b}[0m"),
            (NameStyle::Symlink, "\u{1b}[36mname\u{1b}[0m"),
            (NameStyle::Junction, "\u{1b}[35mname\u{1b}[0m"),
            (NameStyle::Executable, "\u{1b}[1;32mname\u{1b}[0m"),
            (NameStyle::Socket, "\u{1b}[1;35mname\u{1b}[0m"),
            (NameStyle::Fifo, "\u{1b}[33mname\u{1b}[0m"),
            (NameStyle::CharDevice, "\u{1b}[1;33mname\u{1b}[0m"),
            (NameStyle::BlockDevice, "\u{1b}[1;33mname\u{1b}[0m"),
        ] {
            assert_eq!(colorize_name("name", style, false), expected);
        }

        assert_eq!(
            colorize_name("name", NameStyle::Junction, true),
            "\u{1b}[2;35mname\u{1b}[0m"
        );
    });
}

#[test]
fn test_windows_permissions_validate_only_for_long_format() {
    let params = Params {
        permissions: PermissionDisplay::Octal,
        ..Params::default()
    };
    assert!(validate_params(&params).is_ok());
    assert!(
        validate_params(&Params {
            long_format: true,
            ..params
        })
        .is_err()
    );
}

#[test]
fn test_windows_attribute_text_is_readable() {
    assert_eq!(attribute_text(0, AttributeDisplay::Long), "Normal");
    assert_eq!(
        attribute_text(0x0000_0003, AttributeDisplay::Long),
        "ReadOnly, Hidden"
    );
    assert_eq!(
        attribute_text(0x0000_2020, AttributeDisplay::Long),
        "Archive, NotIndexed"
    );
    assert_eq!(
        attribute_text(0x8000_0000, AttributeDisplay::Long),
        "Unknown(0x80000000)"
    );
}

#[test]
fn test_windows_attribute_text_handles_recall_and_structural_bits() {
    assert_eq!(
        attribute_text(0x0004_0000 | 0x0040_0000, AttributeDisplay::Long,),
        "EA, RecallOnDataAccess"
    );
    assert_eq!(
        attribute_text(0x0000_04D0, AttributeDisplay::Long),
        "Normal"
    );
}

#[test]
fn test_windows_short_attribute_text_maps_every_position() {
    let cases = [
        (0x0000_0001, 0, 'R'),
        (0x0000_0002, 1, 'H'),
        (0x0000_0004, 2, 'S'),
        (0x0000_0020, 3, 'A'),
        (0x0000_0100, 4, 'T'),
        (0x0000_0200, 5, 'P'),
        (0x0000_0800, 6, 'C'),
        (0x0000_1000, 7, 'O'),
        (0x0000_2000, 8, 'N'),
        (0x0000_4000, 9, 'E'),
        (0x0000_8000, 10, 'I'),
        (0x0001_0000, 11, 'V'),
        (0x0002_0000, 12, 'B'),
        (0x0004_0000, 13, 'X'),
        (0x0008_0000, 14, 'Q'),
        (0x0010_0000, 15, 'G'),
        (0x0040_0000, 16, 'F'),
    ];

    for (attribute, position, letter) in cases {
        let mut expected = ['-'; 17];
        expected[position] = letter;
        assert_eq!(
            attribute_text(attribute, AttributeDisplay::Short),
            expected.iter().collect::<String>()
        );
    }
}

#[test]
fn test_windows_short_attribute_text_combines_and_preserves_unknown_bits() {
    assert_eq!(
        attribute_text(0, AttributeDisplay::Short),
        "-----------------"
    );
    assert_eq!(
        attribute_text(0x0000_2020, AttributeDisplay::Short),
        "---A----N--------"
    );
    assert_eq!(
        attribute_text(0x8000_0000, AttributeDisplay::Short),
        "----------------- Unknown(0x80000000)"
    );
}

#[test]
fn test_windows_short_attribute_text_ignores_structural_bits() {
    assert_eq!(
        attribute_text(0x0000_04D0, AttributeDisplay::Short),
        "-----------------"
    );
    assert_eq!(
        attribute_text(0x0004_0000, AttributeDisplay::Short),
        "-------------X---"
    );
    assert_eq!(
        attribute_text(0x0040_0000, AttributeDisplay::Short),
        "----------------F"
    );
}

#[test]
fn test_windows_pathext_parser_normalizes_extensions() {
    let extensions = parse_pathext(".exe; .Cmd;PS1");
    assert!(extensions.contains("EXE"));
    assert!(extensions.contains("CMD"));
    assert!(extensions.contains("PS1"));
}

#[test]
fn test_windows_sorting_is_case_insensitive_then_deterministic() {
    assert_eq!(
        compare_entry_names(OsStr::new("Alpha"), OsStr::new("alpha")),
        Ordering::Less
    );
    assert_eq!(
        compare_entry_names(OsStr::new("alpha"), OsStr::new("Beta")),
        Ordering::Less
    );
}

#[test]
fn test_windows_sorting_falls_back_to_utf16_ordering() {
    assert_eq!(
        compare_result_ordering(Some(0), &[0xD800], &[0xD801]),
        Ordering::Less
    );
    assert_eq!(compare_result_ordering(None, &[2], &[1]), Ordering::Greater);
}

#[test]
fn test_windows_reparse_classification_is_conservative() {
    assert_eq!(
        reparse_file_type(Some(0xA000_0003), false),
        LongFormatFileType::Junction
    );
    assert_eq!(
        reparse_file_type(Some(0xA000_000C), true),
        LongFormatFileType::SymlinkDirectory
    );
    assert_eq!(
        reparse_file_type(Some(0xA000_000C), false),
        LongFormatFileType::SymlinkFile
    );
    assert_eq!(
        reparse_file_type(Some(0xDEAD_BEEF), false),
        LongFormatFileType::ReparsePoint
    );
    assert_eq!(
        reparse_file_type(None, false),
        LongFormatFileType::ReparsePoint
    );
}

#[test]
fn test_windows_non_reparse_classification_handles_all_metadata_states() {
    assert_eq!(
        non_reparse_file_type(true, false, false),
        LongFormatFileType::Directory
    );
    assert_eq!(
        non_reparse_file_type(false, true, false),
        LongFormatFileType::Symlink
    );
    assert_eq!(
        non_reparse_file_type(false, false, true),
        LongFormatFileType::Regular
    );
    assert_eq!(
        non_reparse_file_type(false, false, false),
        LongFormatFileType::Unknown
    );
}

#[test]
fn test_windows_normalizes_nt_and_unc_prefixes() {
    assert_eq!(
        normalize_path(PathBuf::from(r"\??\C:\work\entry")),
        PathBuf::from(r"C:\work\entry")
    );
    assert_eq!(
        normalize_path(PathBuf::from(r"\\?\UNC\server\share\entry")),
        PathBuf::from(r"\\server\share\entry")
    );
}

#[test]
fn test_windows_reparse_queries_use_extended_paths() {
    let to_wide = |value: &str| {
        OsStr::new(value)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>()
    };

    assert_eq!(
        extended_find_path(Path::new(r"C:\work\entry")),
        Some(to_wide(r"\\?\C:\work\entry"))
    );
    assert_eq!(
        extended_find_path(Path::new(r"\\server\share\entry")),
        Some(to_wide(r"\\?\UNC\server\share\entry"))
    );
    assert_eq!(
        extended_find_path(Path::new(r"\\?\C:\work\entry")),
        Some(to_wide(r"\\?\C:\work\entry"))
    );
    assert_eq!(
        extended_find_path(Path::new(r"\??\C:\work\entry")),
        Some(to_wide(r"\\?\C:\work\entry"))
    );
}

#[test]
fn test_windows_extended_path_rejects_relative_path_without_current_directory()
{
    assert_eq!(
        extended_find_path_with_current_dir(Path::new("relative"), None),
        None
    );
}

#[test]
fn test_windows_junction_source_uses_junction_style() {
    with_color_output_enabled(|| {
        let display = format_symlink_display_name_with_dim(
            "junction",
            Path::new("junction"),
            Ok(PathBuf::from("target")),
            &Params::default(),
            NameStyle::Junction,
            false,
        );
        assert!(display.contains("\u{1b}[35mjunction"));
    });
}

#[test]
fn test_windows_long_link_text_marks_missing_and_invalid_targets() {
    let temp_dir = tempdir().unwrap();
    let source = temp_dir.path().join("link");
    let params = Params {
        long_format: true,
        ..Params::default()
    };
    let broken = format_symlink_display_name_with_dim(
        "link",
        &source,
        Ok(PathBuf::from("missing")),
        &params,
        NameStyle::Symlink,
        false,
    );
    let unresolved = format_symlink_display_name_with_dim(
        "link",
        &source,
        Ok(PathBuf::from("invalid\0target")),
        &params,
        NameStyle::Symlink,
        false,
    );

    assert!(broken.contains("[Broken Link]"));
    assert!(unresolved.contains("[Target Unresolved]"));
}

#[test]
fn test_windows_directory_symlink_uses_symlink_style() {
    let temp_dir = tempdir().unwrap();
    let file = temp_dir.path().join("file.txt");
    fs::write(&file, "file").unwrap();
    let metadata = fs::symlink_metadata(&file).unwrap();
    let classification = EntryClassification {
        file_type: LongFormatFileType::SymlinkDirectory,
        hidden: false,
        display_as_directory: true,
        group_with_directories: true,
        may_recurse: false,
        may_render_link_target: true,
    };

    assert_eq!(
        crate::platform::name_style(&file, &metadata, classification),
        NameStyle::Symlink
    );
}

#[test]
fn test_windows_slash_indicator_uses_link_object_state() {
    assert_eq!(slash_indicator_suffix(true), "/");
    assert_eq!(slash_indicator_suffix(false), "");
}

#[test]
fn test_windows_collection_filters_hidden_and_groups_directories() {
    let entry =
        |name, file_type, hidden, group_with_directories| DirectoryEntryData {
            file_name: OsString::from(name),
            path: PathBuf::from(name),
            classification_result: Ok(EntryClassification {
                file_type,
                hidden,
                display_as_directory: group_with_directories,
                group_with_directories,
                may_recurse: group_with_directories,
                may_render_link_target: false,
            }),
        };
    let entries = || {
        vec![
            Ok(entry("visible", LongFormatFileType::Regular, false, false)),
            Ok(entry("hidden", LongFormatFileType::Regular, true, false)),
            Ok(entry(
                "directory",
                LongFormatFileType::Directory,
                false,
                true,
            )),
        ]
    };
    let params = Params {
        dirs_first: true,
        ..Params::default()
    };

    assert_eq!(
        collect_visible_file_names(Path::new("listing"), entries(), &params),
        vec![String::from("directory"), String::from("visible")]
    );
    assert_eq!(
        collect_visible_file_names(
            Path::new("listing"),
            entries(),
            &Params {
                show_all: true,
                ..params
            },
        ),
        vec![
            String::from("directory"),
            String::from("hidden"),
            String::from("visible"),
        ]
    );
}

#[test]
fn test_windows_collection_handles_entry_and_classification_errors() {
    let fallback = DirectoryEntryData {
        file_name: OsString::from("fallback"),
        path: PathBuf::from("fallback"),
        classification_result: Err(io::Error::other("metadata")),
    };
    let entries = vec![Ok(fallback), Err(io::Error::other("directory entry"))];
    let params = Params {
        dirs_first: true,
        ..Params::default()
    };

    assert_eq!(
        collect_visible_file_names(Path::new("."), entries, &params),
        vec![String::from("fallback")]
    );
}

#[test]
fn test_windows_collects_a_direct_file_name() {
    let temp_dir = tempdir().unwrap();
    let file = temp_dir.path().join("direct.txt");
    fs::write(&file, "direct").unwrap();

    assert_eq!(
        crate::utils::file::collect_file_names(&file, &Params::default())
            .unwrap(),
        vec![String::from("direct.txt")]
    );
}

#[test]
fn test_windows_classifies_regular_files_and_directories() {
    let temp_dir = tempdir().unwrap();
    let file = temp_dir.path().join("tool.exe");
    let directory = temp_dir.path().join("folder");
    fs::write(&file, "not a program").unwrap();
    fs::create_dir(&directory).unwrap();

    let file_metadata = fs::symlink_metadata(&file).unwrap();
    let file_classification = classify_entry(&file, &file_metadata);
    assert!(!file_classification.display_as_directory);
    assert_eq!(
        crate::platform::name_style(
            &file,
            &file_metadata,
            file_classification
        ),
        NameStyle::Executable
    );

    let directory_metadata = fs::symlink_metadata(&directory).unwrap();
    let directory_classification =
        classify_entry(&directory, &directory_metadata);
    assert!(directory_classification.display_as_directory);
    assert!(directory_classification.group_with_directories);
    assert!(directory_classification.may_recurse);
}

#[test]
fn test_windows_default_config_path_uses_config_directory() {
    let path = default_config_path().unwrap();
    assert_eq!(
        path.file_name().and_then(|name| name.to_str()),
        Some("config.toml")
    );
    assert_eq!(
        path.parent()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str()),
        Some("lsplus")
    );
}
