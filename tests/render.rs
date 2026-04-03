use lsplus::utils::icons::Icon;
use lsplus::utils::render::{
    build_long_format_table, build_short_format_table,
    terminal_width_or_default,
};
use lsplus::{FileInfo, Params};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
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
        nlink: 1,
        user: String::from("user"),
        group: String::from("group"),
        size,
        mtime,
        item_icon,
        display_name: display_name.to_string(),
        full_path: PathBuf::from(display_name),
    }
}

fn normalized_table(table: prettytable::Table) -> String {
    table.to_string().replace("\r\n", "\n")
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
    assert!(rendered.contains("KB"));
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
    assert!(!rendered.contains("KB"));
    assert!(!rendered.contains(&Icon::RustFile.to_string()));
}

#[test]
fn test_build_short_format_table_uses_single_column_for_narrow_width() {
    let files = [
        test_file_info("界界界.txt", None, 0, SystemTime::now()),
        test_file_info("beta.txt", None, 0, SystemTime::now()),
    ];

    let rendered = normalized_table(build_short_format_table(&files, 8));
    let rows: Vec<_> = rendered
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();

    assert_eq!(rows.len(), 2);
}

#[test]
fn test_build_short_format_table_groups_multiple_files_when_width_allows_it() {
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

    let rendered = normalized_table(build_short_format_table(&files, 80));
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
fn test_build_short_format_table_handles_empty_input() {
    let rendered = normalized_table(build_short_format_table(&[], 80));

    assert!(rendered.trim().is_empty());
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
