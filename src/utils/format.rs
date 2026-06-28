//! Formatting helpers for permissions and file sizes.

/// Convert Unix permission bits into an `rwxrwxrwx` string.
pub fn mode_to_rwx(mode: u32) -> String {
    let mut rwx = String::with_capacity(9);

    rwx.push(permission_char(mode, 0o400, 'r'));
    rwx.push(permission_char(mode, 0o200, 'w'));
    rwx.push(special_execute_char(mode, 0o4000, 0o100, 's', 'S'));

    rwx.push(permission_char(mode, 0o040, 'r'));
    rwx.push(permission_char(mode, 0o020, 'w'));
    rwx.push(special_execute_char(mode, 0o2000, 0o010, 's', 'S'));

    rwx.push(permission_char(mode, 0o004, 'r'));
    rwx.push(permission_char(mode, 0o002, 'w'));
    rwx.push(special_execute_char(mode, 0o1000, 0o001, 't', 'T'));

    rwx
}

/// Format Unix permission and special bits as four octal digits.
pub fn mode_to_octal(mode: u32) -> String {
    format!("{:04o}", mode & 0o7777)
}

fn permission_char(mode: u32, bit: u32, value: char) -> char {
    if mode & bit != 0 { value } else { '-' }
}

fn special_execute_char(
    mode: u32,
    special_bit: u32,
    execute_bit: u32,
    execute_char: char,
    no_execute_char: char,
) -> char {
    match (mode & special_bit != 0, mode & execute_bit != 0) {
        (true, true) => execute_char,
        (true, false) => no_execute_char,
        (false, true) => 'x',
        (false, false) => '-',
    }
}

/// Unit scaling mode for human-readable file sizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeScale {
    /// Scale by powers of 1024 and use GNU-style binary suffixes.
    Binary,
    /// Scale by powers of 1000 and use GNU-style decimal suffixes.
    Decimal,
}

impl SizeScale {
    fn base(self) -> f64 {
        match self {
            Self::Binary => 1024.0,
            Self::Decimal => 1000.0,
        }
    }

    fn units(self) -> [&'static str; 6] {
        match self {
            Self::Binary => ["B", "K", "M", "G", "T", "P"],
            Self::Decimal => ["B", "k", "M", "G", "T", "P"],
        }
    }
}

/// Scale a byte count into a human-readable value and unit.
pub fn human_readable_format(
    size: u64,
    scale: SizeScale,
) -> (f64, &'static str) {
    let units = scale.units();
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= scale.base() && unit_index < units.len() - 1 {
        size /= scale.base();
        unit_index += 1;
    }

    let rounded_size = round_to_display_precision(size);
    if rounded_size >= scale.base() && unit_index < units.len() - 1 {
        size = rounded_size / scale.base();
        unit_index += 1;
    }

    (size, units[unit_index])
}

fn round_to_display_precision(size: f64) -> f64 {
    (size * 10.0).round() / 10.0
}

/// Format a size for display and return the optional unit label.
pub fn show_size(
    size: u64,
    scale: Option<SizeScale>,
) -> (String, &'static str) {
    let Some(scale) = scale else {
        return (size.to_string(), "");
    };

    let (size, unit) = human_readable_format(size, scale);
    if size.fract() == 0.0 {
        (format!("{size:.0}"), unit)
    } else {
        (format!("{size:.1}"), unit)
    }
}
