use crate::common_tests::{
    ColorModeGuard, accentless_params, fixed_time_params, has_ansi,
    plain_permission_params, time_only_params, with_color_output_enabled,
};
use crate::utils::color::LongFormatColorLevel;
use crate::utils::format::mode_to_rwx;
use crate::utils::icons::Icon;
use crate::utils::render::{
    build_long_format_table, directory_header_text, render_short_format_lines,
    size_style_spec_for_color_level, terminal_width_or_default,
};
use crate::{FileInfo, NameStyle, Params, structs::PermissionDisplay};
use colored_text::{ColorMode, Colorize};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use strip_ansi_escapes::strip_str;
use terminal_size::{Height, Width};

fn test_file_info(
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

fn normalized_table(table: prettytable::Table) -> String {
    table.to_string().replace("\r\n", "\n")
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
fn test_build_long_format_table_colors_octal_permissions_subtly() {
    for (env, expected) in [
        (
            [("COLORTERM", Some("truecolor")), ("TERM", None::<&str>)],
            "\u{1b}[38;2;238;204;92m0755\u{1b}[0m",
        ),
        (
            [
                ("COLORTERM", None::<&str>),
                ("TERM", Some("xterm-256color")),
            ],
            "\u{1b}[38;5;221m0755\u{1b}[0m",
        ),
        (
            [("COLORTERM", None::<&str>), ("TERM", Some("xterm"))],
            "\u{1b}[2;33m0755\u{1b}[0m",
        ),
    ] {
        temp_env::with_vars(env, || {
            with_color_output_enabled(|| {
                let mut info =
                    test_file_info("script.sh", None, 12, SystemTime::now());
                info.file_type = String::from("-");
                info.mode_bits = 0o755;
                let params = Params {
                    permissions: PermissionDisplay::Octal,
                    ..fixed_time_params()
                };

                let rendered = normalized_table(build_long_format_table(
                    &[info],
                    &params,
                ));

                assert!(rendered.contains(expected));
                assert!(rendered.contains("\u{1b}[2m-\u{1b}[0m"));
            });
        });
    }
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
fn test_size_style_spec_colors_size_boundaries() {
    let params = Params::default();
    let color_level = LongFormatColorLevel::Named;

    assert_eq!(
        size_style_spec_for_color_level(
            1024 * 1024 - 1,
            &params,
            color_level,
            "r"
        ),
        "r"
    );
    assert_eq!(
        size_style_spec_for_color_level(
            1024 * 1024 - 1,
            &params,
            color_level,
            ""
        ),
        ""
    );
    assert_eq!(
        size_style_spec_for_color_level(
            1024 * 1024,
            &params,
            color_level,
            "r"
        ),
        "rFy"
    );
    assert_eq!(
        size_style_spec_for_color_level(1024 * 1024, &params, color_level, ""),
        "Fy"
    );
    assert_eq!(
        size_style_spec_for_color_level(
            1024 * 1024 * 1024,
            &params,
            color_level,
            "r"
        ),
        "rFrb"
    );
    assert_eq!(
        size_style_spec_for_color_level(
            1024 * 1024 * 1024,
            &params,
            color_level,
            ""
        ),
        "Frb"
    );
}

#[test]
fn test_size_style_spec_omits_size_colors_when_disabled() {
    let params = Params {
        size_colors: false,
        ..Params::default()
    };
    let color_level = LongFormatColorLevel::Named;

    assert_eq!(
        size_style_spec_for_color_level(
            1024 * 1024,
            &params,
            color_level,
            "r"
        ),
        "r"
    );
    assert_eq!(
        size_style_spec_for_color_level(1024 * 1024, &params, color_level, ""),
        ""
    );
}

#[test]
fn test_size_style_spec_omits_size_colors_when_global_color_is_disabled() {
    let params = Params::default();
    let color_level = LongFormatColorLevel::None;

    assert_eq!(
        size_style_spec_for_color_level(
            1024 * 1024,
            &params,
            color_level,
            "r"
        ),
        "r"
    );
    assert_eq!(
        size_style_spec_for_color_level(1024 * 1024, &params, color_level, ""),
        ""
    );
}

#[test]
fn test_build_long_format_table_colors_time_buckets() {
    temp_env::with_vars(
        [("COLORTERM", None::<&str>), ("TERM", Some("xterm"))],
        || {
            with_color_output_enabled(|| {
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
                        now.checked_sub(Duration::from_secs(
                            15 * 24 * 60 * 60,
                        ))
                        .unwrap(),
                    ),
                    test_file_info(
                        "old.txt",
                        None,
                        12,
                        now.checked_sub(Duration::from_secs(
                            400 * 24 * 60 * 60,
                        ))
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
        },
    );
}

#[test]
fn test_build_long_format_table_uses_truecolor_for_time_when_supported() {
    temp_env::with_vars(
        [("COLORTERM", Some("truecolor")), ("TERM", None::<&str>)],
        || {
            with_color_output_enabled(|| {
                let now = SystemTime::now();
                let files = [
                    test_file_info("fresh.txt", None, 12, now),
                    test_file_info(
                        "old.txt",
                        None,
                        12,
                        now.checked_sub(Duration::from_secs(
                            400 * 24 * 60 * 60,
                        ))
                        .unwrap(),
                    ),
                ];
                let params = time_only_params();

                let rendered =
                    normalized_table(build_long_format_table(&files, &params));

                assert!(rendered.contains("\u{1b}[38;2;255;209;102m"));
                assert!(rendered.contains("\u{1b}[38;2;150;103;38m"));
            });
        },
    );
}

#[test]
fn test_build_long_format_table_uses_ansi_256_for_time_when_supported() {
    temp_env::with_vars(
        [
            ("COLORTERM", None::<&str>),
            ("TERM", Some("xterm-256color")),
        ],
        || {
            with_color_output_enabled(|| {
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
                        now.checked_sub(Duration::from_secs(
                            14 * 24 * 60 * 60,
                        ))
                        .unwrap(),
                    ),
                    test_file_info(
                        "year.txt",
                        None,
                        12,
                        now.checked_sub(Duration::from_secs(
                            31 * 24 * 60 * 60,
                        ))
                        .unwrap(),
                    ),
                    test_file_info(
                        "older.txt",
                        None,
                        12,
                        now.checked_sub(Duration::from_secs(
                            400 * 24 * 60 * 60,
                        ))
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
            });
        },
    );
}

#[test]
fn test_build_long_format_table_colors_future_time_truecolor() {
    temp_env::with_vars(
        [("COLORTERM", Some("truecolor")), ("TERM", None::<&str>)],
        || {
            with_color_output_enabled(|| {
                let info = test_file_info(
                    "future.txt",
                    None,
                    12,
                    SystemTime::now()
                        .checked_add(Duration::from_secs(60 * 60))
                        .unwrap(),
                );
                let params = time_only_params();

                let rendered = normalized_table(build_long_format_table(
                    &[info],
                    &params,
                ));

                assert!(rendered.contains("\u{1b}[38;2;220;80;70m"));
            });
        },
    );
}

#[test]
fn test_build_long_format_table_colors_future_time_ansi_256() {
    temp_env::with_vars(
        [
            ("COLORTERM", None::<&str>),
            ("TERM", Some("xterm-256color")),
        ],
        || {
            with_color_output_enabled(|| {
                let info = test_file_info(
                    "future.txt",
                    None,
                    12,
                    SystemTime::now()
                        .checked_add(Duration::from_secs(60 * 60))
                        .unwrap(),
                );
                let params = time_only_params();

                let rendered = normalized_table(build_long_format_table(
                    &[info],
                    &params,
                ));

                assert!(rendered.contains("\u{1b}[1;38;5;203m"));
            });
        },
    );
}

#[test]
fn test_build_long_format_table_colors_future_time_named_ansi() {
    temp_env::with_vars(
        [("COLORTERM", None::<&str>), ("TERM", Some("xterm"))],
        || {
            with_color_output_enabled(|| {
                let info = test_file_info(
                    "future.txt",
                    None,
                    12,
                    SystemTime::now()
                        .checked_add(Duration::from_secs(60 * 60))
                        .unwrap(),
                );
                let params = time_only_params();

                let rendered = normalized_table(build_long_format_table(
                    &[info],
                    &params,
                ));

                assert!(rendered.contains("\u{1b}[1;31m"));
            });
        },
    );
}

#[test]
fn test_build_long_format_table_omits_ansi_256_time_when_color_disabled() {
    temp_env::with_vars(
        [
            ("COLORTERM", None::<&str>),
            ("TERM", Some("xterm-256color")),
        ],
        || {
            let _guard = ColorModeGuard::set(ColorMode::Never);
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
    temp_env::with_vars(
        [("COLORTERM", None::<&str>), ("TERM", Some("xterm"))],
        || {
            with_color_output_enabled(|| {
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

                let rendered = normalized_table(build_long_format_table(
                    &[info],
                    &params,
                ));

                assert!(rendered.contains("\u{1b}[1;31m"));
                assert!(!rendered.contains("\u{1b}[33m"));
            });
        },
    );
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

    assert_eq!(short_row, " plain.txt  ");
}

#[test]
fn test_render_short_format_lines_handles_empty_input() {
    let rendered = normalized_lines(render_short_format_lines(&[], 80));

    assert!(rendered.trim().is_empty());
}

#[test]
fn test_render_short_format_lines_style_directory_padding_when_enabled() {
    with_color_output_enabled(|| {
        let mut dir =
            test_file_info("alpha/", Some(Icon::Folder), 0, SystemTime::now());
        dir.name_style = NameStyle::Directory;

        let lines = render_short_format_lines(&[dir], 80);

        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains(&format!(
            "{} \u{1b}[34malpha/  \u{1b}[0m",
            Icon::Folder
        )));
    });
}

#[test]
fn test_render_short_format_lines_keep_plain_output_when_color_disabled() {
    let _guard = ColorModeGuard::set(ColorMode::Never);
    let mut dir =
        test_file_info("alpha/", Some(Icon::Folder), 0, SystemTime::now());
    dir.name_style = NameStyle::Directory;

    let lines = render_short_format_lines(&[dir], 80);

    assert_eq!(lines, vec![format!(" {} alpha/  ", Icon::Folder)]);
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
