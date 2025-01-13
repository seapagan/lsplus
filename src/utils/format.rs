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

    rwx
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_show_size_human_readable() {
        assert_eq!(show_size(1024, true), ("1.0".to_string(), "KB"));
        assert_eq!(show_size(1048576, true), ("1.0".to_string(), "MB"));
        assert_eq!(show_size(1073741824, true), ("1.0".to_string(), "GB"));
        assert_eq!(show_size(1099511627776, true), ("1.0".to_string(), "TB"));
        assert_eq!(show_size(1125899906842624, true), ("1.0".to_string(), "PB"));
    }

    #[test]
    fn test_show_size_non_human_readable() {
        assert_eq!(show_size(1024, false), ("1024".to_string(), ""));
        assert_eq!(show_size(1048576, false), ("1048576".to_string(), ""));
        assert_eq!(show_size(1073741824, false), ("1073741824".to_string(), ""));
        assert_eq!(show_size(1099511627776, false), ("1099511627776".to_string(), ""));
        assert_eq!(show_size(1125899906842624, false), ("1125899906842624".to_string(), ""));
    }
}
