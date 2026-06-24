//! Formatting helpers for permissions and file sizes.

/// Convert Unix permission bits into an `rwxrwxrwx` string.
pub fn mode_to_rwx(mode: u32) -> String {
    let mut rwx = String::new();
    let perms = [
        (mode & 0o400, 'r'),
        (mode & 0o200, 'w'),
        (mode & 0o100, 'x'), // Owner
        (mode & 0o040, 'r'),
        (mode & 0o020, 'w'),
        (mode & 0o010, 'x'), // Group
        (mode & 0o004, 'r'),
        (mode & 0o002, 'w'),
        (mode & 0o001, 'x'), // Others
    ];

    for (bit, chr) in perms.iter() {
        if *bit != 0 {
            rwx.push(*chr);
        } else {
            rwx.push('-');
        }
    }

    let mut chars: Vec<char> = rwx.chars().collect();
    chars[2] = special_execute_char(mode, 0o4000, 0o100, 's', 'S');
    chars[5] = special_execute_char(mode, 0o2000, 0o010, 's', 'S');
    chars[8] = special_execute_char(mode, 0o1000, 0o001, 't', 'T');

    chars.into_iter().collect()
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

/// Scale a byte count into the largest binary unit below 1024.
pub fn human_readable_format(size: u64) -> (f64, &'static str) {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    (size, UNITS[unit_index])
}

/// Format a size for display and return the optional unit label.
pub fn show_size(size: u64, human_readable: bool) -> (String, &'static str) {
    if human_readable {
        let (size, unit) = human_readable_format(size);
        if size.fract() == 0.0 {
            (format!("{:.0}", size), unit)
        } else {
            (format!("{:.1}", size), unit)
        }
    } else {
        (size.to_string(), "")
    }
}
