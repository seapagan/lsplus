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

/// Shortens a filename by replacing the middle with '...' if it exceeds max_width
/// Preserves the file extension and at least 3 characters of the filename
pub fn shorten_filename(filename: &str, max_width: usize) -> String {
    if filename.len() <= max_width {
        return filename.to_string();
    }

    // Split filename and extension
    let (name, ext) = match filename.rfind('.') {
        Some(pos) => (&filename[..pos], &filename[pos..]),
        None => (filename, ""),
    };

    // Ensure we have enough space for at least 3 chars + ... + extension
    let min_width = 6 + ext.len(); // 3 chars + "..." + extension
    if max_width < min_width {
        return filename[..max_width].to_string();
    }

    // Calculate how many characters we can keep from start and end of name
    let available_width = max_width - 3 - ext.len(); // subtract "..." and extension
    let start_chars = available_width / 2;
    let end_chars = available_width - start_chars;

    format!(
        "{}{}{}{}",
        &name[..start_chars],
        "...",
        &name[name.len() - end_chars..],
        ext
    )
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

        // Test show_size with human readable format
        let (size, unit) = show_size(2560, true);
        assert_eq!(size, "2.5");
        assert_eq!(unit, "KB");

        let (size, unit) = show_size(1024, true);
        assert_eq!(size, "1");
        assert_eq!(unit, "KB");

        // Test non-human readable format
        let (size, unit) = show_size(2560, false);
        assert_eq!(size, "2560");
        assert_eq!(unit, "");
    }

    #[test]
    fn test_format_size_extreme() {
        // Test extremely large sizes
        let (size, unit) = human_readable_format(1024 * 1024 * 1024 * 1024);
        assert_eq!(format!("{:.1} {}", size, unit), "1.0 TB");

        let (size, unit) =
            human_readable_format(1024 * 1024 * 1024 * 1024 * 1024);
        assert_eq!(format!("{:.1} {}", size, unit), "1.0 PB");

        // Test exact boundary cases
        let (size, unit) = human_readable_format(1024);
        assert_eq!(format!("{:.1} {}", size, unit), "1.0 KB");

        let (size, unit) = human_readable_format(1024 * 1024);
        assert_eq!(format!("{:.1} {}", size, unit), "1.0 MB");

        // Test just under boundary cases
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
        // Test no permissions
        assert_eq!(mode_to_rwx(0o000), "---------");
        // Test all permissions
        assert_eq!(mode_to_rwx(0o777), "rwxrwxrwx");
        // Test mixed permissions
        assert_eq!(mode_to_rwx(0o750), "rwxr-x---");
    }

    #[test]
    fn test_mode_to_rwx_edge_cases() {
        // Test no permissions
        assert_eq!(mode_to_rwx(0o0000), "---------");

        // Test all permissions
        assert_eq!(mode_to_rwx(0o0777), "rwxrwxrwx");

        // Test write-only (unusual case)
        assert_eq!(mode_to_rwx(0o0222), "-w--w--w-");

        // Test execute-only (unusual case)
        assert_eq!(mode_to_rwx(0o0111), "--x--x--x");
    }

    #[test]
    fn test_shorten_filename() {
        // Test no shortening needed
        assert_eq!(shorten_filename("short.txt", 20), "short.txt");

        // Test basic shortening (balanced between start and end)
        assert_eq!(
            shorten_filename("verylongfilename.txt", 15),
            format!("very{}name.txt", "...")
        );

        // Test with no extension (balanced between start and end)
        assert_eq!(
            shorten_filename("verylongfilename", 10),
            format!("ver{}name", "...")
        );

        // Test with very short max width
        assert_eq!(shorten_filename("long.txt", 5), "long.");

        // Test with exactly max width
        assert_eq!(shorten_filename("exact.txt", 9), "exact.txt");

        // Test with hidden file (balanced between start and end)
        assert_eq!(
            shorten_filename(".longconfigfile.conf", 15),
            format!(".lo{}file.conf", "...")
        );
    }
}
