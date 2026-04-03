use lsplus::utils::format::{human_readable_format, mode_to_rwx, show_size};

#[test]
fn test_format_size() {
    let (size, unit) = human_readable_format(0);
    assert_eq!(format!("{:.1} {}", size, unit), "0.0 B");

    let (size, unit) = human_readable_format(1023);
    assert_eq!(format!("{:.1} {}", size, unit), "1023.0 B");

    let (size, unit) = human_readable_format(1024);
    assert_eq!(format!("{:.1} {}", size, unit), "1.0 KB");

    let (size, unit) = human_readable_format(1024 * 1024);
    assert_eq!(format!("{:.1} {}", size, unit), "1.0 MB");

    let (size, unit) = human_readable_format(1024 * 1024 * 1024);
    assert_eq!(format!("{:.1} {}", size, unit), "1.0 GB");

    let (size, unit) = human_readable_format(1024 * 1024 * 1024 * 1024);
    assert_eq!(format!("{:.1} {}", size, unit), "1.0 TB");
}

#[test]
fn test_format_size_partial() {
    let (size, unit) = human_readable_format(1536);
    assert_eq!(format!("{:.1} {}", size, unit), "1.5 KB");

    let (size, unit) = human_readable_format(1024 * 1024 * 3 / 2);
    assert_eq!(format!("{:.1} {}", size, unit), "1.5 MB");

    let (size, unit) = human_readable_format(1024 * 1024 * 1024 * 5 / 2);
    assert_eq!(format!("{:.1} {}", size, unit), "2.5 GB");

    let (size, unit) = show_size(2560, true);
    assert_eq!(size, "2.5");
    assert_eq!(unit, "KB");

    let (size, unit) = show_size(1024, true);
    assert_eq!(size, "1");
    assert_eq!(unit, "KB");

    let (size, unit) = show_size(2560, false);
    assert_eq!(size, "2560");
    assert_eq!(unit, "");
}

#[test]
fn test_format_size_extreme() {
    let (size, unit) = human_readable_format(1024 * 1024 * 1024 * 1024);
    assert_eq!(format!("{:.1} {}", size, unit), "1.0 TB");

    let (size, unit) = human_readable_format(1024 * 1024 * 1024 * 1024 * 1024);
    assert_eq!(format!("{:.1} {}", size, unit), "1.0 PB");

    let (size, unit) = human_readable_format(1024);
    assert_eq!(format!("{:.1} {}", size, unit), "1.0 KB");

    let (size, unit) = human_readable_format(1024 * 1024);
    assert_eq!(format!("{:.1} {}", size, unit), "1.0 MB");

    let (size, unit) = human_readable_format(1023);
    assert_eq!(format!("{:.1} {}", size, unit), "1023.0 B");

    let (size, unit) = human_readable_format(1024 * 1024 - 1);
    assert_eq!(format!("{:.1} {}", size, unit), "1024.0 KB");
}

#[test]
fn test_format_mode() {
    assert_eq!(mode_to_rwx(0o755), "rwxr-xr-x");
    assert_eq!(mode_to_rwx(0o644), "rw-r--r--");
    assert_eq!(mode_to_rwx(0o777), "rwxrwxrwx");
}

#[test]
fn test_format_mode_permissions() {
    assert_eq!(mode_to_rwx(0o000), "---------");
    assert_eq!(mode_to_rwx(0o777), "rwxrwxrwx");
    assert_eq!(mode_to_rwx(0o750), "rwxr-x---");
}

#[test]
fn test_mode_to_rwx_edge_cases() {
    assert_eq!(mode_to_rwx(0o0000), "---------");
    assert_eq!(mode_to_rwx(0o0777), "rwxrwxrwx");
    assert_eq!(mode_to_rwx(0o0222), "-w--w--w-");
    assert_eq!(mode_to_rwx(0o0111), "--x--x--x");
}
