use crate::common_tests::ColorModeGuard;
use crate::structs::PermissionDisplay;
use crate::utils::icons::Icon;
use crate::utils::render::build_long_format_table;
use crate::{FileInfo, NameStyle, Params};
use colored_text::ColorMode;
use std::path::PathBuf;
use std::time::SystemTime;

fn windows_file_info() -> FileInfo {
    FileInfo {
        file_type: String::from("j"),
        mode: String::from("Hidden, RecallOnOpen"),
        mode_bits: 0,
        nlink: 0,
        user: String::new(),
        group: String::new(),
        size: 1_536,
        mtime: SystemTime::UNIX_EPOCH,
        item_icon: Some(Icon::Junction),
        short_name: String::from("junction"),
        display_name: String::from("junction"),
        name_style: NameStyle::Junction,
        dimmed: false,
        full_path: PathBuf::from("junction"),
    }
}

#[test]
fn test_windows_long_table_uses_native_columns() {
    let _guard = ColorModeGuard::set(ColorMode::Never);
    let params = Params {
        long_format: true,
        header: true,
        human_readable: true,
        ..Params::default()
    };

    let rendered =
        build_long_format_table(&[windows_file_info()], &params).to_string();

    assert!(rendered.contains("Type"));
    assert!(rendered.contains("Attributes"));
    assert!(rendered.contains("Size"));
    assert!(rendered.contains("Date Modified"));
    assert!(rendered.contains("Name"));
    assert!(rendered.contains("Hidden, RecallOnOpen"));
    assert!(rendered.contains("junction"));
    assert!(rendered.contains(&Icon::Junction.to_string()));
    assert!(!rendered.contains("Permissions"));
    assert!(!rendered.contains("Links"));
    assert!(!rendered.contains("User"));
    assert!(!rendered.contains("Group"));
}

#[test]
fn test_windows_long_table_omits_disabled_columns() {
    let _guard = ColorModeGuard::set(ColorMode::Never);
    let params = Params {
        long_format: true,
        header: true,
        no_icons: true,
        permissions: PermissionDisplay::None,
        ..Params::default()
    };

    let rendered =
        build_long_format_table(&[windows_file_info()], &params).to_string();

    assert!(rendered.contains("Type"));
    assert!(rendered.contains("Size"));
    assert!(rendered.contains("Date Modified"));
    assert!(rendered.contains("Name"));
    assert!(!rendered.contains("Attributes"));
    assert!(!rendered.contains(&Icon::Junction.to_string()));
}
