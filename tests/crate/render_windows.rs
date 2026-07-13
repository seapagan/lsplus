use crate::common_tests::{ColorModeGuard, with_color_environment};
use crate::structs::{AttributeDisplay, PermissionDisplay};
use crate::utils::icons::Icon;
use crate::utils::render::{
    build_long_format_table, render_short_format_lines,
};
use crate::{FileInfo, NameStyle, Params};
use colored_text::ColorMode;
use std::path::PathBuf;
use std::time::SystemTime;

fn windows_file_info() -> FileInfo {
    FileInfo {
        file_type: String::from("j"),
        mode: String::from("Hidden, EA"),
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

fn windows_file_info_with_type(file_type: &str) -> FileInfo {
    let mut info = windows_file_info();
    info.file_type = String::from(file_type);
    info
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
    assert!(rendered.contains("Hidden, EA"));
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
        attributes: AttributeDisplay::Short,
        ..Params::default()
    };

    let rendered =
        build_long_format_table(&[windows_file_info()], &params).to_string();

    assert!(rendered.contains("Type"));
    assert!(rendered.contains("Size"));
    assert!(rendered.contains("Date Modified"));
    assert!(rendered.contains("Name"));
    assert!(!rendered.contains("Attributes"));
    assert!(!rendered.contains("Hidden, EA"));
    assert!(!rendered.contains(&Icon::Junction.to_string()));
}

#[test]
fn test_windows_compact_attributes_align_with_headers() {
    let _guard = ColorModeGuard::set(ColorMode::Never);
    let mut first = windows_file_info();
    first.mode = String::from("---A----N--------");
    first.display_name = String::from("first");
    let mut second = windows_file_info();
    second.mode = String::from("-----------------");
    second.display_name = String::from("second");
    let params = Params {
        long_format: true,
        header: true,
        no_icons: true,
        attributes: AttributeDisplay::Short,
        ..Params::default()
    };

    let rendered =
        build_long_format_table(&[first, second], &params).to_string();
    let first_line = rendered
        .lines()
        .find(|line| line.contains("first"))
        .unwrap();
    let second_line = rendered
        .lines()
        .find(|line| line.contains("second"))
        .unwrap();

    assert!(rendered.contains("Attributes"));
    assert_eq!(first_line.find("first"), second_line.find("second"));
}

#[test]
fn test_windows_minimal_attributes_use_short_header_and_align() {
    let _guard = ColorModeGuard::set(ColorMode::Never);
    let mut first = windows_file_info();
    first.mode = String::from("R--A");
    first.display_name = String::from("first");
    let mut second = windows_file_info();
    second.mode = String::from("----");
    second.display_name = String::from("second");
    let params = Params {
        long_format: true,
        header: true,
        no_icons: true,
        attributes: AttributeDisplay::Minimal,
        ..Params::default()
    };

    let rendered =
        build_long_format_table(&[first, second], &params).to_string();
    let first_line = rendered
        .lines()
        .find(|line| line.contains("first"))
        .unwrap();
    let second_line = rendered
        .lines()
        .find(|line| line.contains("second"))
        .unwrap();

    assert!(rendered.contains("Attr"));
    assert!(!rendered.contains("Attributes"));
    assert_eq!(first_line.find("first"), second_line.find("second"));
}

#[test]
fn test_windows_long_table_colors_native_type_markers() {
    with_color_environment(None, None, ColorMode::Always, || {
        let params = Params {
            long_format: true,
            no_icons: true,
            ..Params::default()
        };
        let rendered = build_long_format_table(
            &[
                windows_file_info_with_type("j"),
                windows_file_info_with_type("L"),
                windows_file_info_with_type("r"),
            ],
            &params,
        )
        .to_string();

        assert!(rendered.contains("\u{1b}[35mj"));
        assert!(rendered.contains("\u{1b}[36mL"));
        assert!(rendered.contains("\u{1b}[2mr"));
    });
}

#[test]
fn test_windows_short_format_colors_junction_name() {
    with_color_environment(None, None, ColorMode::Always, || {
        let rendered = render_short_format_lines(&[windows_file_info()], 80);

        assert_eq!(rendered.len(), 1);
        assert_eq!(
            rendered[0],
            format!("{} \u{1b}[35mjunction\u{1b}[0m", Icon::Junction)
        );
    });
}
