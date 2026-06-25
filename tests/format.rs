use lsplus::utils::format::{
    SizeScale, human_readable_format, mode_to_rwx, show_size,
};

#[test]
fn test_format_size() {
    let (size, unit) = human_readable_format(0, SizeScale::Binary);
    assert_eq!(format!("{size:.1} {unit}"), "0.0 B");

    let (size, unit) = human_readable_format(1023, SizeScale::Binary);
    assert_eq!(format!("{size:.1} {unit}"), "1023.0 B");

    let (size, unit) = human_readable_format(1024, SizeScale::Binary);
    assert_eq!(format!("{size:.1} {unit}"), "1.0 K");

    let (size, unit) = human_readable_format(1024 * 1024, SizeScale::Binary);
    assert_eq!(format!("{size:.1} {unit}"), "1.0 M");

    let (size, unit) =
        human_readable_format(1024 * 1024 * 1024, SizeScale::Binary);
    assert_eq!(format!("{size:.1} {unit}"), "1.0 G");

    let (size, unit) =
        human_readable_format(1024 * 1024 * 1024 * 1024, SizeScale::Binary);
    assert_eq!(format!("{size:.1} {unit}"), "1.0 T");
}

#[test]
fn test_format_size_partial() {
    let (size, unit) = human_readable_format(1536, SizeScale::Binary);
    assert_eq!(format!("{size:.1} {unit}"), "1.5 K");

    let (size, unit) =
        human_readable_format(1024 * 1024 * 3 / 2, SizeScale::Binary);
    assert_eq!(format!("{size:.1} {unit}"), "1.5 M");

    let (size, unit) =
        human_readable_format(1024 * 1024 * 1024 * 5 / 2, SizeScale::Binary);
    assert_eq!(format!("{size:.1} {unit}"), "2.5 G");

    let (size, unit) = show_size(2560, Some(SizeScale::Binary));
    assert_eq!(size, "2.5");
    assert_eq!(unit, "K");

    let (size, unit) = show_size(1024, Some(SizeScale::Binary));
    assert_eq!(size, "1");
    assert_eq!(unit, "K");

    let (size, unit) = show_size(2560, None);
    assert_eq!(size, "2560");
    assert_eq!(unit, "");
}

#[test]
fn test_format_size_decimal() {
    let (size, unit) = human_readable_format(999, SizeScale::Decimal);
    assert_eq!(format!("{size:.1} {unit}"), "999.0 B");

    let (size, unit) = human_readable_format(1000, SizeScale::Decimal);
    assert_eq!(format!("{size:.1} {unit}"), "1.0 k");

    let (size, unit) = human_readable_format(1500, SizeScale::Decimal);
    assert_eq!(format!("{size:.1} {unit}"), "1.5 k");

    let (size, unit) = human_readable_format(1_000_000, SizeScale::Decimal);
    assert_eq!(format!("{size:.1} {unit}"), "1.0 M");

    let (size, unit) = show_size(1000, Some(SizeScale::Decimal));
    assert_eq!(size, "1");
    assert_eq!(unit, "k");
}

#[test]
fn test_format_size_extreme() {
    let (size, unit) =
        human_readable_format(1024 * 1024 * 1024 * 1024, SizeScale::Binary);
    assert_eq!(format!("{size:.1} {unit}"), "1.0 T");

    let (size, unit) = human_readable_format(
        1024 * 1024 * 1024 * 1024 * 1024,
        SizeScale::Binary,
    );
    assert_eq!(format!("{size:.1} {unit}"), "1.0 P");

    let (size, unit) = human_readable_format(1024, SizeScale::Binary);
    assert_eq!(format!("{size:.1} {unit}"), "1.0 K");

    let (size, unit) = human_readable_format(1024 * 1024, SizeScale::Binary);
    assert_eq!(format!("{size:.1} {unit}"), "1.0 M");

    let (size, unit) = human_readable_format(1023, SizeScale::Binary);
    assert_eq!(format!("{size:.1} {unit}"), "1023.0 B");

    let (size, unit) =
        human_readable_format(1024 * 1024 - 1, SizeScale::Binary);
    assert_eq!(format!("{size:.1} {unit}"), "1024.0 K");
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

#[test]
fn test_mode_to_rwx_special_bits() {
    assert_eq!(mode_to_rwx(0o4755), "rwsr-xr-x");
    assert_eq!(mode_to_rwx(0o4644), "rwSr--r--");
    assert_eq!(mode_to_rwx(0o2755), "rwxr-sr-x");
    assert_eq!(mode_to_rwx(0o2644), "rw-r-Sr--");
    assert_eq!(mode_to_rwx(0o1755), "rwxr-xr-t");
    assert_eq!(mode_to_rwx(0o1644), "rw-r--r-T");
    assert_eq!(mode_to_rwx(0o7777), "rwsrwsrwt");
}
