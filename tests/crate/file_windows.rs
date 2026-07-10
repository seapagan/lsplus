use crate::platform::{
    LongColumn, LongFormatLayoutOptions, attribute_text, classify_entry,
    compare_entry_names, default_config_path, long_format_layout,
    parse_pathext, validate_params,
};
use crate::structs::PermissionDisplay;
use crate::{Params, structs::NameStyle};
use std::cmp::Ordering;
use std::ffi::OsStr;
use std::fs;
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
    assert_eq!(attribute_text(0), "Normal");
    assert!(attribute_text(0x0000_0003).contains("ReadOnly, Hidden"));
    assert!(attribute_text(0x8000_0000).contains("Unknown(0x80000000)"));
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
