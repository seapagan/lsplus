use crate::common_tests::{
    ColorModeGuard, accentless_params, fixed_time_params, has_ansi,
    plain_permission_params, time_only_params, with_color_environment,
    with_color_output_enabled,
};
use crate::utils::icons::Icon;
use crate::utils::render::{
    SizeCellStyle, build_long_format_table,
    build_long_format_table_with_name_prefixes, directory_header_text,
    render_short_format_lines, render_short_single_column_lines,
    short_output_uses_grid, size_style_for_color_level,
    terminal_width_or_default,
};
use crate::{
    FileInfo, NameStyle, Params, ShortFormat, structs::PermissionDisplay,
};
use colored_text::{ColorLevel, ColorMode, Colorize};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use strip_ansi_escapes::strip_str;
use terminal_size::{Height, Width};
use unicode_width::UnicodeWidthStr;

pub(crate) fn test_file_info(
    display_name: &str,
    item_icon: Option<Icon>,
    size: u64,
    mtime: SystemTime,
) -> FileInfo {
    FileInfo {
        file_type: String::from("-"),
        mode: String::from("rw-r--r--"),
        mode_bits: 0o644,
        nlink: 1,
        user: String::from("user"),
        group: String::from("group"),
        size,
        mtime,
        item_icon,
        short_name: display_name.to_string(),
        display_name: display_name.to_string(),
        name_style: NameStyle::Plain,
        dimmed: false,
        full_path: PathBuf::from(display_name),
    }
}

fn normalized_lines(lines: Vec<String>) -> String {
    lines.join("\n")
}

pub(crate) fn normalized_table(table: impl std::fmt::Display) -> String {
    table.to_string().replace("\r\n", "\n")
}

pub(crate) fn visible_column_start(row: &str, needle: &str) -> usize {
    let byte_start = row.find(needle).unwrap_or_else(|| {
        panic!("needle {needle:?} not found in row {row:?}")
    });
    UnicodeWidthStr::width(strip_str(&row[..byte_start]).as_str())
}

pub(crate) fn visible_column_end(row: &str, needle: &str) -> usize {
    visible_column_start(row, needle)
        + UnicodeWidthStr::width(strip_str(needle).as_str())
}

#[test]
fn test_build_long_format_table_omits_permissions() {
    let mut info = test_file_info("private", None, 12, SystemTime::now());
    info.file_type = String::from("-");
    info.mode = String::from("---------");
    info.mode_bits = 0;
    let params = Params {
        permissions: PermissionDisplay::None,
        ..plain_permission_params()
    };

    let rendered = normalized_table(build_long_format_table(&[info], &params));

    assert!(rendered.contains("private"));
    assert!(!rendered.contains("---------"));
    assert!(!rendered.contains("0000"));
}

#[test]
fn test_build_long_format_table_includes_units_and_icons() {
    let info = test_file_info(
        "example.rs",
        Some(Icon::RustFile),
        2 * 1024,
        SystemTime::now(),
    );
    let params = Params {
        human_readable: true,
        ..Params::default()
    };

    let rendered = normalized_table(build_long_format_table(&[info], &params));

    assert!(rendered.contains("example.rs"));
    assert!(rendered.contains("2"));
    assert!(rendered.contains("K"));
    assert!(rendered.contains(&Icon::RustFile.to_string()));
}

#[test]
fn test_build_long_format_table_uses_fuzzy_time_when_requested() {
    let info = test_file_info(
        "aged.txt",
        None,
        128,
        SystemTime::now()
            .checked_sub(Duration::from_secs(2 * 60 * 60))
            .unwrap(),
    );
    let params = Params {
        fuzzy_time: true,
        ..Params::default()
    };

    let rendered = normalized_table(build_long_format_table(&[info], &params));

    assert!(rendered.contains("aged.txt"));
    assert!(rendered.contains("2 hours ago"));
}

#[test]
fn test_build_long_format_table_omits_optional_units_and_icons() {
    let info = test_file_info("plain.txt", None, 12, SystemTime::now());

    let rendered =
        normalized_table(build_long_format_table(&[info], &Params::default()));

    assert!(rendered.contains("plain.txt"));
    assert!(rendered.contains("12"));
    assert!(!rendered.contains("K"));
    assert!(!rendered.contains(&Icon::RustFile.to_string()));
}

#[test]
fn test_build_long_format_table_omits_header_by_default() {
    let info = test_file_info("plain.txt", None, 12, SystemTime::now());

    let rendered =
        normalized_table(build_long_format_table(&[info], &Params::default()));

    assert!(!rendered.contains("Permissions"));
    assert!(!rendered.contains("Date Modified"));
}

#[test]
fn test_build_long_format_table_omits_header_for_empty_rows() {
    let params = Params {
        header: true,
        ..Params::default()
    };

    let rendered = normalized_table(build_long_format_table(&[], &params));

    assert!(!rendered.contains("Permissions"));
    assert!(!rendered.contains("Name"));
}

#[test]
fn test_build_long_format_table_aligns_names_with_blank_icon_cells() {
    let files = [
        test_file_info("plain.txt", None, 12, SystemTime::now()),
        test_file_info(
            "example.rs",
            Some(Icon::RustFile),
            12,
            SystemTime::now(),
        ),
    ];

    let rendered =
        normalized_table(build_long_format_table(&files, &Params::default()));
    let plain_row = rendered
        .lines()
        .find(|line| line.contains("plain.txt"))
        .unwrap();
    let icon_row = rendered
        .lines()
        .find(|line| line.contains("example.rs"))
        .unwrap();
    let icon = Icon::RustFile.to_string();

    assert_eq!(
        visible_column_start(plain_row, "plain.txt"),
        visible_column_start(icon_row, "example.rs")
    );
    assert_eq!(
        visible_column_start(icon_row, "example.rs"),
        visible_column_end(icon_row, &icon) + 2
    );
}

#[test]
fn test_build_long_format_table_with_name_prefixes_uses_same_alignment() {
    let files = [
        test_file_info("plain.txt", None, 12, SystemTime::now()),
        test_file_info(
            "example.rs",
            Some(Icon::RustFile),
            12,
            SystemTime::now(),
        ),
    ];
    let prefixed = [(&files[0], "|-- "), (&files[1], "`-- ")];

    let rendered =
        normalized_table(build_long_format_table_with_name_prefixes(
            prefixed,
            &Params::default(),
        ));
    let plain_row = rendered
        .lines()
        .find(|line| line.contains("plain.txt"))
        .unwrap();
    let icon_row = rendered
        .lines()
        .find(|line| line.contains("example.rs"))
        .unwrap();

    assert_eq!(
        visible_column_start(plain_row, "|-- plain.txt"),
        visible_column_start(icon_row, "`-- example.rs")
    );
}

#[test]
fn test_build_long_format_table_colors_special_file_types() {
    with_color_output_enabled(|| {
        let mut pipe = test_file_info("pipe", None, 0, SystemTime::now());
        pipe.file_type = String::from("p");
        let mut socket = test_file_info("socket", None, 0, SystemTime::now());
        socket.file_type = String::from("s");
        let mut char_device =
            test_file_info("char", None, 0, SystemTime::now());
        char_device.file_type = String::from("c");
        let mut block_device =
            test_file_info("block", None, 0, SystemTime::now());
        block_device.file_type = String::from("b");
        let mut unknown =
            test_file_info("unknown", None, 0, SystemTime::now());
        unknown.file_type = String::from("?");

        let rendered = normalized_table(build_long_format_table(
            &[pipe, socket, char_device, block_device, unknown],
            &fixed_time_params(),
        ));

        assert!(rendered.contains("\u{1b}[33mp\u{1b}[0m"));
        assert!(rendered.contains("\u{1b}[1;35ms\u{1b}[0m"));
        assert!(rendered.contains("\u{1b}[1;33mc\u{1b}[0m"));
        assert!(rendered.contains("\u{1b}[1;33mb\u{1b}[0m"));
        assert!(rendered.contains("\u{1b}[2m?\u{1b}[0m"));
    });
}

#[test]
fn test_build_long_format_table_colors_size_boundaries() {
    with_color_output_enabled(|| {
        let files = [
            test_file_info(
                "small.bin",
                None,
                1024 * 1024 - 1,
                SystemTime::now(),
            ),
            test_file_info("large.bin", None, 1024 * 1024, SystemTime::now()),
            test_file_info(
                "huge.bin",
                None,
                1024 * 1024 * 1024,
                SystemTime::now(),
            ),
        ];
        let params = Params {
            human_readable: true,
            ..plain_permission_params()
        };

        let rendered =
            normalized_table(build_long_format_table(&files, &params));
        let stripped = strip_str(&rendered);
        let rows: Vec<_> = stripped
            .lines()
            .filter(|line| line.contains(".bin"))
            .collect();

        assert_eq!(rows.len(), 3);
        assert!(rows[0].contains("1 M"));
        assert!(rows[1].contains("1 M"));
        assert!(rows[2].contains("1 G"));
        assert!(!stripped.contains("1024 K"));
    });
}

#[test]
fn test_build_long_format_table_aligns_colored_size_cells() {
    with_color_output_enabled(|| {
        let files = [
            test_file_info("plain.bin", None, 808, SystemTime::now()),
            test_file_info(
                "large.bin",
                None,
                8 * 1024 * 1024,
                SystemTime::now(),
            ),
            test_file_info(
                "huge.bin",
                None,
                57 * 1024 * 1024 * 1024,
                SystemTime::now(),
            ),
        ];
        let params = Params {
            human_readable: true,
            ..plain_permission_params()
        };

        let rendered =
            normalized_table(build_long_format_table(&files, &params));
        let stripped = strip_str(&rendered);
        let rows: Vec<_> = stripped
            .lines()
            .filter(|line| line.contains(".bin"))
            .collect();

        assert_eq!(rows.len(), 3);
        let plain_size_end = rows[0].find("808").unwrap() + "808".len();
        let large_size_end = rows[1].find('8').unwrap() + "8".len();
        let huge_size_end = rows[2].find("57").unwrap() + "57".len();
        let plain_unit_start = rows[0].find("B  ").unwrap();
        let large_unit_start = rows[1].find("M ").unwrap();
        let huge_unit_start = rows[2].find("G ").unwrap();

        assert_eq!(plain_size_end, large_size_end);
        assert_eq!(large_size_end, huge_size_end);
        assert_eq!(plain_unit_start, large_unit_start);
        assert_eq!(large_unit_start, huge_unit_start);
        assert!(rows[0].contains("808 B"));
    });
}

#[test]
fn test_build_long_format_table_omits_size_colors_when_disabled() {
    with_color_output_enabled(|| {
        let info =
            test_file_info("large.bin", None, 1024 * 1024, SystemTime::now());
        let params = Params {
            human_readable: true,
            ..accentless_params()
        };

        let rendered =
            normalized_table(build_long_format_table(&[info], &params));

        assert!(rendered.contains("1 M"));
        assert!(!rendered.contains("\u{1b}[33m1\u{1b}[0m"));
        assert!(!rendered.contains("\u{1b}[33mM\u{1b}[0m"));
    });
}

#[test]
fn test_size_style_for_color_level_colors_size_boundaries() {
    let params = Params::default();
    let color_level = ColorLevel::Ansi16;

    assert_eq!(
        size_style_for_color_level(
            1024 * 1024 - 1,
            &params,
            color_level,
            true
        ),
        SizeCellStyle::PlainRight
    );
    assert_eq!(
        size_style_for_color_level(
            1024 * 1024 - 1,
            &params,
            color_level,
            false
        ),
        SizeCellStyle::Plain
    );
    assert_eq!(
        size_style_for_color_level(1024 * 1024, &params, color_level, true),
        SizeCellStyle::LargeRight
    );
    assert_eq!(
        size_style_for_color_level(1024 * 1024, &params, color_level, false),
        SizeCellStyle::Large
    );
    assert_eq!(
        size_style_for_color_level(
            1024 * 1024 * 1024,
            &params,
            color_level,
            true
        ),
        SizeCellStyle::HugeRight
    );
    assert_eq!(
        size_style_for_color_level(
            1024 * 1024 * 1024,
            &params,
            color_level,
            false
        ),
        SizeCellStyle::Huge
    );
}

#[test]
fn test_size_style_for_color_level_omits_size_colors_when_disabled() {
    let params = Params {
        size_colors: false,
        ..Params::default()
    };
    let color_level = ColorLevel::Ansi16;

    assert_eq!(
        size_style_for_color_level(1024 * 1024, &params, color_level, true),
        SizeCellStyle::PlainRight
    );
    assert_eq!(
        size_style_for_color_level(1024 * 1024, &params, color_level, false),
        SizeCellStyle::Plain
    );
}

#[test]
fn test_build_long_format_table_keeps_future_time_plain_without_color() {
    let _guard = ColorModeGuard::set(ColorMode::Never);
    let info = test_file_info(
        "future.txt",
        None,
        12,
        SystemTime::now()
            .checked_add(Duration::from_secs(60 * 60))
            .unwrap(),
    );
    let params = Params {
        no_color: true,
        ..Params::default()
    };

    let rendered = normalized_table(build_long_format_table(&[info], &params));

    assert!(rendered.contains("future.txt"));
    assert_eq!(strip_str(&rendered), rendered);
}

#[test]
fn test_build_long_format_table_colors_time_gradient_segments() {
    let now = SystemTime::now();
    let files = [
        test_file_info(
            "days.txt",
            None,
            12,
            now.checked_sub(Duration::from_secs(2 * 24 * 60 * 60))
                .unwrap(),
        ),
        test_file_info(
            "weeks.txt",
            None,
            12,
            now.checked_sub(Duration::from_secs(14 * 24 * 60 * 60))
                .unwrap(),
        ),
        test_file_info(
            "months.txt",
            None,
            12,
            now.checked_sub(Duration::from_secs(60 * 24 * 60 * 60))
                .unwrap(),
        ),
    ];

    with_color_environment(None, Some("truecolor"), ColorMode::Always, || {
        let rendered = normalized_table(build_long_format_table(
            &files,
            &time_only_params(),
        ));

        for name in ["days.txt", "weeks.txt", "months.txt"] {
            let row =
                rendered.lines().find(|line| line.contains(name)).unwrap();
            assert!(has_ansi(row));
        }
    });
}

#[test]
fn test_render_short_format_styles_special_name_types() {
    let styles = [
        (NameStyle::Socket, "socket", "\u{1b}[1;35msocket"),
        (NameStyle::Fifo, "pipe", "\u{1b}[33mpipe"),
        (NameStyle::CharDevice, "char", "\u{1b}[1;33mchar"),
        (NameStyle::BlockDevice, "block", "\u{1b}[1;33mblock"),
    ];

    with_color_output_enabled(|| {
        for (name_style, name, expected) in styles {
            let mut info = test_file_info(name, None, 0, SystemTime::now());
            info.name_style = name_style;

            let rendered =
                normalized_lines(render_short_format_lines(&[info], 80));

            assert!(rendered.contains(expected));
        }
    });
}

#[test]
fn test_size_style_for_color_level_omits_size_colors_when_global_color_is_disabled()
 {
    let params = Params::default();
    let color_level = ColorLevel::NoColor;

    assert_eq!(
        size_style_for_color_level(1024 * 1024, &params, color_level, true),
        SizeCellStyle::PlainRight
    );
    assert_eq!(
        size_style_for_color_level(1024 * 1024, &params, color_level, false),
        SizeCellStyle::Plain
    );
}

#[test]
fn test_build_long_format_table_colors_time_buckets() {
    with_color_environment(Some("xterm"), None, ColorMode::Always, || {
        let now = SystemTime::now();
        let files = [
            test_file_info("fresh.txt", None, 12, now),
            test_file_info(
                "week.txt",
                None,
                12,
                now.checked_sub(Duration::from_secs(2 * 24 * 60 * 60))
                    .unwrap(),
            ),
            test_file_info(
                "month.txt",
                None,
                12,
                now.checked_sub(Duration::from_secs(15 * 24 * 60 * 60))
                    .unwrap(),
            ),
            test_file_info(
                "old.txt",
                None,
                12,
                now.checked_sub(Duration::from_secs(400 * 24 * 60 * 60))
                    .unwrap(),
            ),
        ];
        let params = time_only_params();

        let rendered =
            normalized_table(build_long_format_table(&files, &params));

        assert!(rendered.contains("\u{1b}[1;33m"));
        assert!(rendered.contains("\u{1b}[33m"));
        assert!(rendered.contains("\u{1b}[2;33m"));
    });
}

#[test]
fn test_build_long_format_table_uses_truecolor_for_time_when_supported() {
    with_color_environment(None, Some("truecolor"), ColorMode::Always, || {
        let now = SystemTime::now();
        let files = [
            test_file_info("fresh.txt", None, 12, now),
            test_file_info(
                "old.txt",
                None,
                12,
                now.checked_sub(Duration::from_secs(400 * 24 * 60 * 60))
                    .unwrap(),
            ),
        ];
        let params = time_only_params();

        let rendered =
            normalized_table(build_long_format_table(&files, &params));

        assert!(rendered.contains("\u{1b}[38;2;255;209;102m"));
        assert!(rendered.contains("\u{1b}[38;2;150;103;38m"));
    });
}

#[test]
fn test_build_long_format_table_uses_ansi_256_for_time_when_supported() {
    with_color_environment(
        Some("xterm-256color"),
        None,
        ColorMode::Always,
        || {
            let now = SystemTime::now();
            let files = [
                test_file_info("fresh.txt", None, 12, now),
                test_file_info(
                    "week.txt",
                    None,
                    12,
                    now.checked_sub(Duration::from_secs(2 * 24 * 60 * 60))
                        .unwrap(),
                ),
                test_file_info(
                    "month.txt",
                    None,
                    12,
                    now.checked_sub(Duration::from_secs(14 * 24 * 60 * 60))
                        .unwrap(),
                ),
                test_file_info(
                    "year.txt",
                    None,
                    12,
                    now.checked_sub(Duration::from_secs(31 * 24 * 60 * 60))
                        .unwrap(),
                ),
                test_file_info(
                    "older.txt",
                    None,
                    12,
                    now.checked_sub(Duration::from_secs(400 * 24 * 60 * 60))
                        .unwrap(),
                ),
            ];
            let params = time_only_params();

            let rendered =
                normalized_table(build_long_format_table(&files, &params));

            assert!(rendered.contains("\u{1b}[1;38;5;222m"));
            assert!(rendered.contains("\u{1b}[38;5;221m"));
            assert!(rendered.contains("\u{1b}[38;5;178m"));
            assert!(rendered.contains("\u{1b}[38;5;136m"));
            assert!(rendered.contains("\u{1b}[38;5;130m"));
            assert!(!rendered.contains("\u{1b}[38;2;"));
        },
    );
}

#[test]
fn test_build_long_format_table_colors_future_time_truecolor() {
    with_color_environment(None, Some("truecolor"), ColorMode::Always, || {
        let info = test_file_info(
            "future.txt",
            None,
            12,
            SystemTime::now()
                .checked_add(Duration::from_secs(60 * 60))
                .unwrap(),
        );
        let params = time_only_params();

        let rendered =
            normalized_table(build_long_format_table(&[info], &params));

        assert!(rendered.contains("\u{1b}[38;2;220;80;70m"));
    });
}

#[test]
fn test_build_long_format_table_colors_future_time_ansi_256() {
    with_color_environment(
        Some("xterm-256color"),
        None,
        ColorMode::Always,
        || {
            let info = test_file_info(
                "future.txt",
                None,
                12,
                SystemTime::now()
                    .checked_add(Duration::from_secs(60 * 60))
                    .unwrap(),
            );
            let params = time_only_params();

            let rendered =
                normalized_table(build_long_format_table(&[info], &params));

            assert!(rendered.contains("\u{1b}[1;38;5;203m"));
        },
    );
}

#[test]
fn test_build_long_format_table_colors_future_time_named_ansi() {
    with_color_environment(Some("xterm"), None, ColorMode::Always, || {
        let info = test_file_info(
            "future.txt",
            None,
            12,
            SystemTime::now()
                .checked_add(Duration::from_secs(60 * 60))
                .unwrap(),
        );
        let params = time_only_params();

        let rendered =
            normalized_table(build_long_format_table(&[info], &params));

        assert!(rendered.contains("\u{1b}[1;31m"));
    });
}

#[test]
fn test_build_long_format_table_omits_ansi_256_time_when_color_disabled() {
    with_color_environment(
        Some("xterm-256color"),
        None,
        ColorMode::Never,
        || {
            let info =
                test_file_info("future.txt", None, 12, SystemTime::now());

            let rendered = normalized_table(build_long_format_table(
                &[info],
                &Params::default(),
            ));

            assert!(!has_ansi(&rendered));
        },
    );
}

#[test]
fn test_build_long_format_table_uses_fixed_time_color_when_gradient_disabled()
{
    with_color_output_enabled(|| {
        let info = test_file_info("fresh.txt", None, 12, SystemTime::now());
        let params = accentless_params();

        let rendered =
            normalized_table(build_long_format_table(&[info], &params));

        assert!(rendered.contains("\u{1b}[33m"));
        assert!(!rendered.contains("\u{1b}[1;93m"));
        assert!(!rendered.contains("\u{1b}[93m"));
        assert!(!rendered.contains("\u{1b}[38;2;"));
        assert!(!rendered.contains("\u{1b}[38;5;"));
    });
}

#[test]
fn test_build_long_format_table_colors_future_time_when_gradient_disabled() {
    with_color_environment(Some("xterm"), None, ColorMode::Always, || {
        let info = test_file_info(
            "future.txt",
            None,
            12,
            SystemTime::now()
                .checked_add(Duration::from_secs(60 * 60))
                .unwrap(),
        );
        let params = Params {
            fuzzy_time: false,
            ..accentless_params()
        };

        let rendered =
            normalized_table(build_long_format_table(&[info], &params));

        assert!(rendered.contains("\u{1b}[1;31m"));
        assert!(!rendered.contains("\u{1b}[33m"));
    });
}

#[test]
fn test_build_long_format_table_is_plain_when_color_is_disabled() {
    let _guard = ColorModeGuard::set(ColorMode::Never);
    let info = test_file_info("plain.txt", None, 12, SystemTime::now());

    let rendered =
        normalized_table(build_long_format_table(&[info], &Params::default()));

    assert!(!has_ansi(&rendered));
}

#[test]
fn test_build_long_format_table_does_not_pad_short_rows_to_widest_name() {
    let files = [
        test_file_info("plain.txt", None, 12, SystemTime::now()),
        test_file_info(
            "this-is-a-very-long-filename-that-should-not-pad-other-rows.txt",
            None,
            12,
            SystemTime::now(),
        ),
    ];

    let rendered =
        normalized_table(build_long_format_table(&files, &Params::default()));
    let short_row = rendered
        .lines()
        .find(|line| line.contains("plain.txt"))
        .unwrap();

    assert!(short_row.ends_with("plain.txt"));
}

#[test]
fn test_render_short_format_lines_uses_single_column_for_narrow_width() {
    let files = [
        test_file_info("界界界.txt", None, 0, SystemTime::now()),
        test_file_info("beta.txt", None, 0, SystemTime::now()),
    ];

    let rendered = normalized_lines(render_short_format_lines(&files, 8));
    let rows: Vec<_> = rendered
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();

    assert_eq!(rows.len(), 2);
}

#[test]
fn test_render_short_format_lines_groups_multiple_files_when_width_allows_it()
{
    let files = [
        test_file_info("alpha.txt", None, 0, SystemTime::now()),
        test_file_info("beta.txt", None, 0, SystemTime::now()),
        test_file_info(
            "gamma.txt",
            Some(Icon::RustFile),
            0,
            SystemTime::now(),
        ),
    ];

    let rendered = normalized_lines(render_short_format_lines(&files, 80));
    let rows: Vec<_> = rendered
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();

    assert_eq!(rows.len(), 1);
    assert!(rows[0].contains("alpha.txt"));
    assert!(rows[0].contains("beta.txt"));
    assert!(rows[0].contains("gamma.txt"));
    assert!(rows[0].contains(&Icon::RustFile.to_string()));
}

#[test]
fn test_render_short_format_lines_does_not_pad_short_rows_to_widest_name() {
    let files = [
        test_file_info("plain.txt", None, 0, SystemTime::now()),
        test_file_info(
            "this-is-a-very-long-filename.txt",
            None,
            0,
            SystemTime::now(),
        ),
    ];

    let rendered = normalized_lines(render_short_format_lines(&files, 20));
    let short_row = rendered
        .lines()
        .find(|line| line.contains("plain.txt"))
        .unwrap();

    assert_eq!(short_row, "plain.txt");
}

#[test]
fn test_render_short_format_lines_uses_gnu_vertical_order_and_column_widths() {
    let files = ["a", "bbbbb", "cc", "dddddddd", "eee", "fffffff"]
        .map(|name| test_file_info(name, None, 0, SystemTime::now()));

    assert_eq!(
        render_short_format_lines(&files, 20),
        vec![
            String::from("a      dddddddd"),
            String::from("bbbbb  eee"),
            String::from("cc     fffffff"),
        ]
    );
}

#[test]
fn test_render_short_format_lines_handles_incomplete_final_column() {
    let files = ["a", "b", "c", "dddd", "e"]
        .map(|name| test_file_info(name, None, 0, SystemTime::now()));

    assert_eq!(
        render_short_format_lines(&files, 8),
        vec![
            String::from("a  dddd"),
            String::from("b  e"),
            String::from("c")
        ]
    );
}

#[test]
fn test_render_short_format_lines_respects_exact_width_boundary() {
    let files = ["a", "bbb"]
        .map(|name| test_file_info(name, None, 0, SystemTime::now()));

    assert_eq!(render_short_format_lines(&files, 6), vec!["a  bbb"]);
    assert_eq!(render_short_format_lines(&files, 5), vec!["a", "bbb"]);
}

#[test]
fn test_render_short_format_lines_keeps_wide_names_untruncated() {
    let files = [test_file_info("界界界.txt", None, 0, SystemTime::now())];

    assert_eq!(
        render_short_format_lines(&files, 0),
        vec![String::from("界界界.txt")]
    );
}

#[test]
fn test_render_short_format_lines_handles_empty_input() {
    let rendered = normalized_lines(render_short_format_lines(&[], 80));

    assert!(rendered.trim().is_empty());
}

#[test]
fn test_render_short_format_lines_style_directory_when_enabled() {
    with_color_output_enabled(|| {
        let mut dir =
            test_file_info("alpha/", Some(Icon::Folder), 0, SystemTime::now());
        dir.name_style = NameStyle::Directory;

        let lines = render_short_format_lines(&[dir], 80);

        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0],
            format!("{} \u{1b}[34malpha/\u{1b}[0m", Icon::Folder)
        );
    });
}

#[test]
fn test_render_short_format_lines_keep_plain_output_when_color_disabled() {
    let _guard = ColorModeGuard::set(ColorMode::Never);
    let mut dir =
        test_file_info("alpha/", Some(Icon::Folder), 0, SystemTime::now());
    dir.name_style = NameStyle::Directory;

    let lines = render_short_format_lines(&[dir], 80);

    assert_eq!(lines, vec![format!("{} alpha/", Icon::Folder)]);
}

#[test]
fn test_render_short_format_lines_ignores_ansi_when_measuring_columns() {
    with_color_output_enabled(|| {
        let mut directory = test_file_info("界/", None, 0, SystemTime::now());
        directory.name_style = NameStyle::Directory;
        let plain = test_file_info("x", None, 0, SystemTime::now());

        let lines = render_short_format_lines(&[directory, plain], 7);

        assert_eq!(lines.len(), 1);
        assert_eq!(strip_str(&lines[0]), "界/  x");
        assert!(lines[0].contains("\u{1b}[34m界/\u{1b}[0m"));
    });
}

#[test]
fn test_render_short_single_column_lines_has_no_grid_padding() {
    let files = ["alpha", "beta"]
        .map(|name| test_file_info(name, None, 0, SystemTime::now()));

    assert_eq!(
        render_short_single_column_lines(&files),
        vec![String::from("alpha"), String::from("beta")]
    );
}

#[test]
fn test_short_output_uses_grid_for_terminal_or_explicit_vertical_format() {
    assert!(short_output_uses_grid(true, None));
    assert!(short_output_uses_grid(false, Some(ShortFormat::Vertical)));
    assert!(!short_output_uses_grid(false, None));
}

#[test]
fn test_directory_header_text_uses_bold_directory_color_when_enabled() {
    with_color_output_enabled(|| {
        assert_eq!(
            directory_header_text("src"),
            "src".blue().bold().to_string()
        );
    });
}

#[test]
fn test_directory_header_text_keeps_plain_output_when_color_disabled() {
    let _guard = ColorModeGuard::set(ColorMode::Never);

    assert_eq!(directory_header_text("src"), "src");
}

#[test]
fn test_terminal_width_or_default_uses_detected_width() {
    assert_eq!(
        terminal_width_or_default(Some((Width(120), Height(40)))),
        120
    );
}

#[test]
fn test_terminal_width_or_default_falls_back_to_80() {
    assert_eq!(terminal_width_or_default(None), 80);
}
