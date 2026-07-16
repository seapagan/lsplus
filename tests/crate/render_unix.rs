use crate::common_tests::{
    fixed_time_params, plain_permission_params, with_color_environment,
    with_color_output_enabled,
};
use crate::render_tests::{
    normalized_table, test_file_info, visible_column_end, visible_column_start,
};
use crate::utils::format::mode_to_rwx;
use crate::utils::icons::Icon;
use crate::utils::render::{
    build_long_format_table, render_short_format_lines,
};
use crate::{NameStyle, Params, ShortFormat, structs::PermissionDisplay};
use colored_text::ColorMode;
use std::time::{Duration, SystemTime};
use strip_ansi_escapes::strip_str;

#[test]
fn test_render_short_format_lines_preserves_synthetic_dot_names() {
    with_color_output_enabled(|| {
        let mut dot = test_file_info(".", None, 0, SystemTime::now());
        dot.name_style = NameStyle::Directory;
        let mut dotdot = test_file_info("..", None, 0, SystemTime::now());
        dotdot.name_style = NameStyle::Directory;

        let rendered = render_short_format_lines(
            &[dot, dotdot],
            80,
            ShortFormat::Vertical,
        );

        assert_eq!(rendered.len(), 1);
        assert_eq!(strip_str(&rendered[0]), ".  ..");
        assert!(rendered[0].contains("\u{1b}[34m.\u{1b}[0m"));
        assert!(rendered[0].contains("\u{1b}[34m..\u{1b}[0m"));
    });
}

#[test]
fn test_build_long_format_table_adds_header_before_rows() {
    let info = test_file_info("plain.txt", None, 12, SystemTime::now());
    let params = Params {
        header: true,
        no_icons: true,
        ..Params::default()
    };

    let rendered = normalized_table(build_long_format_table(&[info], &params));
    let stripped = strip_str(&rendered);
    let rows: Vec<_> = stripped.lines().collect();

    assert!(rows[0].contains("Permissions"));
    assert!(rows[0].contains("Links"));
    assert!(rows[0].contains("Date Modified"));
    assert!(rows[0].contains("Name"));
    assert!(rows[1].contains("plain.txt"));
}

#[test]
fn test_build_long_format_table_aligns_after_colored_symbolic_permissions() {
    with_color_output_enabled(|| {
        let mut first =
            test_file_info("first.txt", None, 12, SystemTime::now());
        first.file_type = String::from("d");
        first.mode = String::from("rwsr-tS-T");
        first.nlink = 1;

        let mut second =
            test_file_info("second.txt", None, 12, SystemTime::now());
        second.file_type = String::from("-");
        second.mode = String::from("rwxrwxrwx");
        second.nlink = 123_456;

        let rendered = normalized_table(build_long_format_table(
            &[first, second],
            &fixed_time_params(),
        ));
        let rows: Vec<_> = rendered
            .lines()
            .filter(|line| line.contains(".txt"))
            .collect();

        assert_eq!(rows.len(), 2);
        assert!(rows[0].contains("\u{1b}["));
        assert_eq!(
            visible_column_start(rows[0], "user"),
            visible_column_start(rows[1], "user")
        );
        assert_eq!(
            visible_column_start(rows[0], "group"),
            visible_column_start(rows[1], "group")
        );
    });
}

#[test]
fn test_build_long_format_table_colors_columns_when_enabled() {
    with_color_output_enabled(|| {
        let info = test_file_info("plain.txt", None, 12, SystemTime::now());

        let rendered = normalized_table(build_long_format_table(
            &[info],
            &Params::default(),
        ));

        assert!(rendered.contains("\u{1b}[36muser\u{1b}[0m"));
        assert!(rendered.contains("\u{1b}[32mgroup\u{1b}[0m"));
        assert!(rendered.contains("\u{1b}[33m"));
    });
}

#[test]
fn test_build_long_format_table_colors_header_when_enabled() {
    for (term, colorterm, expected) in [
        (
            None,
            Some("truecolor"),
            "\u{1b}[4;38;2;250;128;114mPermissions\u{1b}[0m",
        ),
        (
            Some("xterm-256color"),
            None,
            "\u{1b}[4;38;5;209mPermissions\u{1b}[0m",
        ),
        (Some("xterm"), None, "\u{1b}[4;31mPermissions\u{1b}[0m"),
    ] {
        with_color_environment(term, colorterm, ColorMode::Always, || {
            let info =
                test_file_info("plain.txt", None, 12, SystemTime::now());
            let params = Params {
                header: true,
                no_icons: true,
                ..fixed_time_params()
            };

            let rendered =
                normalized_table(build_long_format_table(&[info], &params));

            assert!(rendered.contains(expected));
        });
    }
}

#[test]
fn test_build_long_format_table_colors_octal_permissions_subtly() {
    for (term, colorterm, expected) in [
        (
            None,
            Some("truecolor"),
            "\u{1b}[38;2;238;204;92m0755\u{1b}[0m",
        ),
        (
            Some("xterm-256color"),
            None,
            "\u{1b}[38;5;221m0755\u{1b}[0m",
        ),
        (Some("xterm"), None, "\u{1b}[2;33m0755\u{1b}[0m"),
    ] {
        with_color_environment(term, colorterm, ColorMode::Always, || {
            let mut info =
                test_file_info("script.sh", None, 12, SystemTime::now());
            info.file_type = String::from("-");
            info.mode_bits = 0o755;
            let params = Params {
                permissions: PermissionDisplay::Octal,
                ..fixed_time_params()
            };

            let rendered =
                normalized_table(build_long_format_table(&[info], &params));

            assert!(rendered.contains(expected));
            assert!(rendered.contains("\u{1b}[2m-\u{1b}[0m"));
        });
    }
}

#[test]
fn test_build_long_format_table_colors_permissions_by_default() {
    with_color_output_enabled(|| {
        let mut info =
            test_file_info("script.sh", None, 12, SystemTime::now());
        info.file_type = String::from("d");
        info.mode = String::from("rwsr-tS-T");

        let rendered = normalized_table(build_long_format_table(
            &[info],
            &fixed_time_params(),
        ));

        assert!(rendered.contains("\u{1b}[34md\u{1b}[0m"));
        assert!(rendered.contains("\u{1b}[32mr\u{1b}[0m"));
        assert!(rendered.contains("\u{1b}[33mw\u{1b}[0m"));
        assert!(rendered.contains("\u{1b}[1;31ms\u{1b}[0m"));
        assert!(rendered.contains("\u{1b}[1;31mt\u{1b}[0m"));
        assert!(rendered.contains("\u{1b}[2mS\u{1b}[0m"));
        assert!(rendered.contains("\u{1b}[2mT\u{1b}[0m"));
    });
}

#[test]
fn test_build_long_format_table_colors_symlink_and_fallback_mode_chars() {
    with_color_output_enabled(|| {
        let mut symlink = test_file_info("link", None, 0, SystemTime::now());
        symlink.file_type = String::from("l");

        let mut fallback =
            test_file_info("fallback", None, 0, SystemTime::now());
        fallback.file_type = String::from("z");
        fallback.mode = String::from("q--------");

        let rendered = normalized_table(build_long_format_table(
            &[symlink, fallback],
            &fixed_time_params(),
        ));

        assert!(rendered.contains("\u{1b}[36ml\u{1b}[0m"));
        assert!(rendered.contains("zq"));
    });
}

#[test]
fn test_build_long_format_table_header_aligns_with_colored_rows() {
    with_color_environment(None, Some("truecolor"), ColorMode::Always, || {
        let mut first =
            test_file_info("first.txt", None, 12, SystemTime::now());
        first.file_type = String::from("d");
        first.mode = String::from("rwsr-tS-T");
        first.nlink = 1;

        let mut second = test_file_info(
            "second.txt",
            Some(Icon::RustFile),
            12,
            SystemTime::now(),
        );
        second.file_type = String::from("-");
        second.mode = String::from("rwxrwxrwx");
        second.nlink = 123_456;

        let params = Params {
            header: true,
            human_readable: true,
            ..fixed_time_params()
        };
        let rendered = normalized_table(build_long_format_table(
            &[first, second],
            &params,
        ));
        let rows: Vec<_> = rendered.lines().collect();

        assert!(rows[0].contains("Permissions"));
        assert!(rows[1].contains("\u{1b}["));
        assert!(
            visible_column_start(rows[0], "User")
                < visible_column_start(rows[0], "Group")
        );
        assert!(
            visible_column_start(rows[0], "Group")
                < visible_column_start(rows[0], "Name")
        );
        assert_eq!(
            visible_column_start(rows[1], "user"),
            visible_column_start(rows[2], "user")
        );
        assert_eq!(
            visible_column_start(rows[1], "group"),
            visible_column_start(rows[2], "group")
        );
        assert_eq!(
            visible_column_start(rows[1], "first.txt"),
            visible_column_start(rows[2], "second.txt")
        );
    });
}

#[test]
fn test_build_long_format_table_header_matches_optional_columns() {
    let info = test_file_info(
        "example.rs",
        Some(Icon::RustFile),
        2 * 1024,
        SystemTime::now(),
    );
    let params = Params {
        header: true,
        human_readable: true,
        permissions: PermissionDisplay::Both,
        ..Params::default()
    };

    let rendered = normalized_table(build_long_format_table(&[info], &params));
    let header = strip_str(&rendered).lines().next().unwrap().to_string();

    assert!(header.contains("Permissions"));
    assert!(header.contains("Octal"));
    assert!(header.contains("Links"));
    assert!(header.contains("User"));
    assert!(header.contains("Group"));
    assert!(header.contains("Size"));
    assert!(header.contains("Date Modified"));
    assert!(header.contains("Name"));
    assert!(!header.contains("Unit"));
}

#[test]
fn test_build_long_format_table_header_omits_disabled_columns() {
    let info = test_file_info("plain.txt", None, 12, SystemTime::now());
    let params = Params {
        header: true,
        no_icons: true,
        permissions: PermissionDisplay::None,
        ..Params::default()
    };

    let rendered = normalized_table(build_long_format_table(&[info], &params));
    let header = strip_str(&rendered).lines().next().unwrap().to_string();

    assert!(!header.contains("Permissions"));
    assert!(!header.contains("Octal"));
    assert!(!header.contains("Unit"));
    assert!(header.contains("Links"));
    assert!(header.contains("Name"));
}

#[test]
fn test_build_long_format_table_header_uses_column_alignment() {
    let mut info = test_file_info(
        "plain.txt",
        None,
        12,
        SystemTime::now()
            .checked_sub(Duration::from_secs(2 * 60 * 60))
            .unwrap(),
    );
    info.nlink = 123_456;
    let params = Params {
        fuzzy_time: true,
        header: true,
        human_readable: true,
        no_icons: true,
        ..plain_permission_params()
    };

    let rendered = normalized_table(build_long_format_table(&[info], &params));
    let rows: Vec<_> = rendered.lines().collect();

    assert_eq!(
        visible_column_start(rows[0], "Links"),
        visible_column_start(rows[1], "123456")
    );
    assert_eq!(
        visible_column_start(rows[0], "User"),
        visible_column_start(rows[1], "user")
    );
    assert_eq!(
        visible_column_start(rows[0], "Group"),
        visible_column_start(rows[1], "group")
    );
    assert_eq!(
        visible_column_start(rows[0], "Name"),
        visible_column_start(rows[1], "plain.txt")
    );
    assert_eq!(
        visible_column_end(rows[0], "Size"),
        visible_column_end(rows[1], "12 B")
    );
    assert_eq!(
        visible_column_end(rows[0], "Date Modified"),
        visible_column_end(rows[1], "2 hours ago")
    );
}

#[test]
fn test_build_long_format_table_keeps_header_plain_when_color_disabled() {
    with_color_output_enabled(|| {
        let info = test_file_info("plain.txt", None, 12, SystemTime::now());
        let params = Params {
            header: true,
            no_color: true,
            no_icons: true,
            ..Params::default()
        };

        let rendered =
            normalized_table(build_long_format_table(&[info], &params));
        let header = rendered
            .lines()
            .find(|line| line.contains("Permissions"))
            .unwrap();

        assert_eq!(strip_str(header), header);
    });
}

#[test]
fn test_build_long_format_table_omits_permission_colors_when_disabled() {
    with_color_output_enabled(|| {
        let mut info =
            test_file_info("script.sh", None, 12, SystemTime::now());
        info.file_type = String::from("d");
        info.mode = String::from("rwxr-x---");

        let rendered = normalized_table(build_long_format_table(
            &[info],
            &plain_permission_params(),
        ));
        let line = rendered
            .lines()
            .find(|line| strip_str(line).contains("script.sh"))
            .unwrap();

        assert!(line.contains("drwxr-x---"));
        assert!(!line.contains("\u{1b}[34md\u{1b}[0m"));
        assert!(!line.contains("\u{1b}[32mr\u{1b}[0m"));
    });
}

#[test]
fn test_build_long_format_table_preserves_left_aligned_links_without_header() {
    let mut first = test_file_info("one.txt", None, 12, SystemTime::now());
    first.nlink = 1;
    let mut second = test_file_info("many.txt", None, 12, SystemTime::now());
    second.nlink = 123_456;
    let params = Params {
        no_icons: true,
        ..plain_permission_params()
    };

    let rendered =
        normalized_table(build_long_format_table(&[first, second], &params));
    let rows: Vec<_> = rendered.lines().collect();

    assert_eq!(
        visible_column_start(rows[0], "1"),
        visible_column_start(rows[1], "123456")
    );
}

#[test]
fn test_build_long_format_table_replaces_symbolic_permissions_with_octal() {
    let mut info = test_file_info("script.sh", None, 12, SystemTime::now());
    info.file_type = String::from("-");
    info.mode = String::from("rwxr-xr-x");
    info.mode_bits = 0o4755;
    let params = Params {
        permissions: PermissionDisplay::Octal,
        ..plain_permission_params()
    };

    let rendered = normalized_table(build_long_format_table(&[info], &params));
    let stripped = strip_str(&rendered);

    assert!(stripped.contains("- 4755  1"));
    assert!(!rendered.contains("-rwxr-xr-x"));
}

#[test]
fn test_build_long_format_table_shows_symbolic_and_octal_permissions() {
    let mut info = test_file_info("sticky", None, 12, SystemTime::now());
    info.file_type = String::from("d");
    info.mode = String::from("rwxrwxrwt");
    info.mode_bits = 0o1777;
    let params = Params {
        permissions: PermissionDisplay::Both,
        ..plain_permission_params()
    };

    let rendered = normalized_table(build_long_format_table(&[info], &params));
    let stripped = strip_str(&rendered);

    assert!(stripped.contains("drwxrwxrwt 1777  1"));
}

#[test]
fn test_build_long_format_table_shows_symbolic_permissions_by_default() {
    let mut info = test_file_info("plain.txt", None, 12, SystemTime::now());
    info.file_type = String::from("-");
    info.mode = String::from("rw-r--r--");
    info.mode_bits = 0o644;

    let rendered = normalized_table(build_long_format_table(
        &[info],
        &plain_permission_params(),
    ));

    assert!(rendered.contains("-rw-r--r--"));
    assert!(!rendered.contains("0644"));
}

#[test]
fn test_build_long_format_table_styles_every_mode_to_rwx_char() {
    with_color_output_enabled(|| {
        let emitted = [
            mode_to_rwx(0o7777),
            mode_to_rwx(0o4644),
            mode_to_rwx(0o1644),
            mode_to_rwx(0o0111),
            mode_to_rwx(0o0000),
        ]
        .join("");
        for value in ['r', 'w', 'x', '-', 's', 'S', 't', 'T'] {
            assert!(emitted.contains(value));
        }

        let mut info =
            test_file_info("script.sh", None, 12, SystemTime::now());
        info.mode = emitted;

        let rendered = normalized_table(build_long_format_table(
            &[info],
            &fixed_time_params(),
        ));

        assert!(rendered.contains("\u{1b}[32mr\u{1b}[0m"));
        assert!(rendered.contains("\u{1b}[33mw\u{1b}[0m"));
        assert!(rendered.contains("\u{1b}[1;31mx\u{1b}[0m"));
        assert!(rendered.contains("\u{1b}[1;31ms\u{1b}[0m"));
        assert!(rendered.contains("\u{1b}[1;31mt\u{1b}[0m"));
        assert!(rendered.contains("\u{1b}[2m-\u{1b}[0m"));
        assert!(rendered.contains("\u{1b}[2mS\u{1b}[0m"));
        assert!(rendered.contains("\u{1b}[2mT\u{1b}[0m"));
    });
}
