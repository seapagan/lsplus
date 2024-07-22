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
