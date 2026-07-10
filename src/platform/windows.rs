//! Windows filesystem metadata interpretation.

use std::cmp::Ordering;
use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::os::windows::fs::{FileTypeExt, MetadataExt};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::SystemTime;

use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
use windows_sys::Win32::Globalization::CompareStringOrdinal;
use windows_sys::Win32::Storage::FileSystem::{
    FILE_ATTRIBUTE_ARCHIVE, FILE_ATTRIBUTE_COMPRESSED,
    FILE_ATTRIBUTE_ENCRYPTED, FILE_ATTRIBUTE_HIDDEN,
    FILE_ATTRIBUTE_INTEGRITY_STREAM, FILE_ATTRIBUTE_NOT_CONTENT_INDEXED,
    FILE_ATTRIBUTE_OFFLINE, FILE_ATTRIBUTE_PINNED, FILE_ATTRIBUTE_READONLY,
    FILE_ATTRIBUTE_REPARSE_POINT, FILE_ATTRIBUTE_SPARSE_FILE,
    FILE_ATTRIBUTE_SYSTEM, FILE_ATTRIBUTE_TEMPORARY, FILE_ATTRIBUTE_UNPINNED,
    FILE_ATTRIBUTE_VIRTUAL, FindClose, FindFirstFileW, WIN32_FIND_DATAW,
};

use crate::platform::{
    EntryClassification, FileDetails, LongColumn, LongFormatFileType,
    LongFormatLayout, LongFormatLayoutOptions,
};
use crate::structs::{NameStyle, Params, PermissionDisplay};

const FILE_ATTRIBUTE_NO_SCRUB_DATA: u32 = 0x0002_0000;
const FILE_ATTRIBUTE_EA: u32 = 0x0004_0000;
const FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS: u32 = 0x0040_0000;
const IO_REPARSE_TAG_MOUNT_POINT: u32 = 0xA000_0003;
const IO_REPARSE_TAG_SYMLINK: u32 = 0xA000_000C;

pub(crate) fn classify_entry(
    path: &Path,
    metadata: &fs::Metadata,
) -> EntryClassification {
    let file_type = metadata_file_type(path, metadata);
    let is_dir = matches!(
        file_type,
        LongFormatFileType::Directory
            | LongFormatFileType::SymlinkDirectory
            | LongFormatFileType::Junction
    );
    let is_link = matches!(
        file_type,
        LongFormatFileType::Symlink
            | LongFormatFileType::SymlinkFile
            | LongFormatFileType::SymlinkDirectory
            | LongFormatFileType::Junction
    );

    EntryClassification {
        file_type,
        hidden: metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0,
        display_as_directory: is_dir,
        group_with_directories: is_dir,
        may_recurse: matches!(file_type, LongFormatFileType::Directory),
        may_render_link_target: is_link,
    }
}

pub(crate) fn metadata_file_type(
    path: &Path,
    metadata: &fs::Metadata,
) -> LongFormatFileType {
    let attributes = metadata.file_attributes();
    if attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return match reparse_tag(path) {
            Some(IO_REPARSE_TAG_MOUNT_POINT) => LongFormatFileType::Junction,
            Some(IO_REPARSE_TAG_SYMLINK) => {
                let file_type = metadata.file_type();
                if file_type.is_symlink_dir() {
                    LongFormatFileType::SymlinkDirectory
                } else {
                    LongFormatFileType::SymlinkFile
                }
            }
            _ => LongFormatFileType::ReparsePoint,
        };
    }

    if metadata.is_dir() {
        LongFormatFileType::Directory
    } else if metadata.file_type().is_symlink() {
        LongFormatFileType::Symlink
    } else if metadata.is_file() {
        LongFormatFileType::Regular
    } else {
        LongFormatFileType::Unknown
    }
}

pub(crate) fn file_details(
    _path: &Path,
    metadata: &fs::Metadata,
    classification: EntryClassification,
) -> FileDetails {
    FileDetails {
        file_type: classification.file_type.as_char().to_string(),
        mode: attribute_text(metadata.file_attributes()),
        mode_bits: 0,
        nlink: 0,
        size: metadata.len(),
        mtime: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
        user: String::new(),
        group: String::new(),
    }
}

pub(crate) fn compare_entry_names(left: &OsStr, right: &OsStr) -> Ordering {
    let left = left.encode_wide().collect::<Vec<_>>();
    let right = right.encode_wide().collect::<Vec<_>>();
    compare_wide(&left, &right, true)
        .then_with(|| compare_wide(&left, &right, false))
}

pub(crate) fn is_executable(path: &Path, _metadata: &fs::Metadata) -> bool {
    let Some(extension) = path.extension() else {
        return false;
    };
    executable_extensions().iter().any(|candidate| {
        extension.to_string_lossy().eq_ignore_ascii_case(candidate)
    })
}

pub(crate) fn name_style(
    path: &Path,
    metadata: &fs::Metadata,
    classification: EntryClassification,
) -> NameStyle {
    match classification.file_type {
        LongFormatFileType::Directory => NameStyle::Directory,
        LongFormatFileType::Symlink
        | LongFormatFileType::SymlinkFile
        | LongFormatFileType::SymlinkDirectory => NameStyle::Symlink,
        LongFormatFileType::Junction => NameStyle::Junction,
        LongFormatFileType::Regular if is_executable(path, metadata) => {
            NameStyle::Executable
        }
        _ => NameStyle::Plain,
    }
}

pub(crate) fn synthetic_dot_entries(
    _params: &Params,
) -> &'static [&'static str] {
    &[]
}

pub(crate) fn validate_params(params: &Params) -> io::Result<()> {
    if params.long_format
        && matches!(
            params.permissions,
            PermissionDisplay::Octal | PermissionDisplay::Both
        )
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Windows does not support octal permission display; use symbolic or none.",
        ));
    }
    Ok(())
}

pub(crate) fn default_config_path() -> Option<PathBuf> {
    let mut path = dirs_next::config_dir()?;
    path.push("lsplus");
    path.push("config.toml");
    Some(path)
}

pub(crate) fn normalize_path(path: PathBuf) -> PathBuf {
    let wide = path.as_os_str().encode_wide().collect::<Vec<_>>();
    const EXTENDED_PREFIX: &[u16] = &[92, 92, 63, 92];
    const NT_PREFIX: &[u16] = &[92, 63, 63, 92];
    const UNC_PREFIX: &[u16] = &[85, 78, 67, 92];
    let prefix_len = if wide.starts_with(EXTENDED_PREFIX) {
        EXTENDED_PREFIX.len()
    } else if wide.starts_with(NT_PREFIX) {
        NT_PREFIX.len()
    } else {
        return path;
    };
    let remainder = &wide[prefix_len..];
    if remainder.starts_with(UNC_PREFIX) {
        let mut normalized = vec![92, 92];
        normalized.extend_from_slice(&remainder[UNC_PREFIX.len()..]);
        return PathBuf::from(OsString::from_wide(&normalized));
    }
    PathBuf::from(OsString::from_wide(remainder))
}

pub(crate) fn long_format_layout(
    options: &LongFormatLayoutOptions,
) -> LongFormatLayout {
    let mut columns = vec![LongColumn::Type];
    if options.permission_display == PermissionDisplay::Symbolic {
        columns.push(LongColumn::Attributes);
    }
    columns.push(LongColumn::Size);
    if options.include_size_unit {
        columns.push(LongColumn::Unit);
    }
    columns.push(LongColumn::Date);
    if options.include_icon {
        columns.push(LongColumn::Icon);
    }
    columns.push(LongColumn::Name);
    LongFormatLayout { columns }
}

fn reparse_tag(path: &Path) -> Option<u32> {
    let wide = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let mut data: WIN32_FIND_DATAW = unsafe { std::mem::zeroed() };
    let handle = unsafe { FindFirstFileW(wide.as_ptr(), &mut data) };
    if handle == INVALID_HANDLE_VALUE {
        return None;
    }
    unsafe { FindClose(handle) };
    Some(data.dwReserved0)
}

fn compare_wide(left: &[u16], right: &[u16], ignore_case: bool) -> Ordering {
    let left_len = i32::try_from(left.len());
    let right_len = i32::try_from(right.len());
    let (Ok(left_len), Ok(right_len)) = (left_len, right_len) else {
        return left.cmp(right);
    };
    let result = unsafe {
        CompareStringOrdinal(
            left.as_ptr(),
            left_len,
            right.as_ptr(),
            right_len,
            ignore_case.into(),
        )
    };
    match result {
        1 => Ordering::Less,
        2 => Ordering::Equal,
        3 => Ordering::Greater,
        _ => left.cmp(right),
    }
}

fn executable_extensions() -> &'static HashSet<String> {
    static EXTENSIONS: OnceLock<HashSet<String>> = OnceLock::new();
    EXTENSIONS.get_or_init(|| {
        let value = std::env::var("PATHEXT")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| String::from(".COM;.EXE;.BAT;.CMD"));
        parse_pathext(&value)
    })
}

pub(crate) fn parse_pathext(value: &str) -> HashSet<String> {
    value
        .split(';')
        .map(str::trim)
        .filter(|extension| !extension.is_empty())
        .map(|extension| {
            extension.trim_start_matches('.').to_ascii_uppercase()
        })
        .collect()
}

pub(crate) fn attribute_text(attributes: u32) -> String {
    let known = [
        (FILE_ATTRIBUTE_READONLY, "ReadOnly"),
        (FILE_ATTRIBUTE_HIDDEN, "Hidden"),
        (FILE_ATTRIBUTE_SYSTEM, "System"),
        (FILE_ATTRIBUTE_ARCHIVE, "Archive"),
        (FILE_ATTRIBUTE_TEMPORARY, "Temporary"),
        (FILE_ATTRIBUTE_SPARSE_FILE, "Sparse"),
        (FILE_ATTRIBUTE_COMPRESSED, "Compressed"),
        (FILE_ATTRIBUTE_OFFLINE, "Offline"),
        (FILE_ATTRIBUTE_NOT_CONTENT_INDEXED, "NotIndexed"),
        (FILE_ATTRIBUTE_ENCRYPTED, "Encrypted"),
        (FILE_ATTRIBUTE_INTEGRITY_STREAM, "IntegrityStream"),
        (FILE_ATTRIBUTE_VIRTUAL, "Virtual"),
        (FILE_ATTRIBUTE_NO_SCRUB_DATA, "NoScrubData"),
        (FILE_ATTRIBUTE_EA, "EA"),
        (FILE_ATTRIBUTE_PINNED, "Pinned"),
        (FILE_ATTRIBUTE_UNPINNED, "Unpinned"),
        (FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS, "RecallOnDataAccess"),
    ];
    let mut values = known
        .iter()
        .filter_map(|(flag, name)| {
            (attributes & flag != 0).then_some((*name).to_string())
        })
        .collect::<Vec<_>>();
    let known_mask = known
        .iter()
        .fold(FILE_ATTRIBUTE_REPARSE_POINT, |mask, (flag, _)| mask | flag);
    let unknown = attributes & !known_mask & !0x0000_0090;
    if unknown != 0 {
        values.push(format!("Unknown(0x{unknown:08X})"));
    }
    if values.is_empty() {
        String::from("Normal")
    } else {
        values.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pathext_normalizes_extensions() {
        let extensions = parse_pathext(".exe; .Cmd;PS1");
        assert!(extensions.contains("EXE"));
        assert!(extensions.contains("CMD"));
        assert!(extensions.contains("PS1"));
    }
}
