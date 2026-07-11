//! Platform-specific filesystem metadata interpretation.

use std::time::SystemTime;

use crate::structs::{NameStyle, PermissionDisplay};

/// Platform-neutral interpretation of one directory entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EntryClassification {
    pub(crate) file_type: LongFormatFileType,
    pub(crate) hidden: bool,
    pub(crate) display_as_directory: bool,
    pub(crate) group_with_directories: bool,
    pub(crate) may_recurse: bool,
    pub(crate) may_render_link_target: bool,
}

/// File types that can appear in long-format output.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LongFormatFileType {
    Directory,
    Regular,
    Symlink,
    #[cfg_attr(unix, allow(dead_code))]
    SymlinkFile,
    #[cfg_attr(unix, allow(dead_code))]
    SymlinkDirectory,
    #[cfg_attr(unix, allow(dead_code))]
    Junction,
    #[cfg_attr(unix, allow(dead_code))]
    ReparsePoint,
    #[cfg_attr(windows, allow(dead_code))]
    Socket,
    #[cfg_attr(windows, allow(dead_code))]
    Fifo,
    #[cfg_attr(windows, allow(dead_code))]
    CharDevice,
    #[cfg_attr(windows, allow(dead_code))]
    BlockDevice,
    Unknown,
}

impl LongFormatFileType {
    pub(crate) fn as_char(self) -> char {
        match self {
            Self::Directory => 'd',
            Self::Regular => '-',
            Self::Symlink | Self::SymlinkFile => 'l',
            Self::SymlinkDirectory => 'L',
            Self::Junction => 'j',
            Self::ReparsePoint => 'r',
            Self::Socket => 's',
            Self::Fifo => 'p',
            Self::CharDevice => 'c',
            Self::BlockDevice => 'b',
            Self::Unknown => '?',
        }
    }
}

/// Return the shared name style for a classified file type.
pub(crate) fn name_style_for_file_type(
    file_type: LongFormatFileType,
    executable: bool,
) -> NameStyle {
    match file_type {
        LongFormatFileType::Symlink
        | LongFormatFileType::SymlinkFile
        | LongFormatFileType::SymlinkDirectory => NameStyle::Symlink,
        LongFormatFileType::Junction => NameStyle::Junction,
        LongFormatFileType::Directory => NameStyle::Directory,
        LongFormatFileType::Socket => NameStyle::Socket,
        LongFormatFileType::Fifo => NameStyle::Fifo,
        LongFormatFileType::CharDevice => NameStyle::CharDevice,
        LongFormatFileType::BlockDevice => NameStyle::BlockDevice,
        LongFormatFileType::Regular if executable => NameStyle::Executable,
        LongFormatFileType::Regular
        | LongFormatFileType::ReparsePoint
        | LongFormatFileType::Unknown => NameStyle::Plain,
    }
}

/// Metadata consumed by the shared rendering pipeline.
pub(crate) struct FileDetails {
    pub(crate) file_type: String,
    pub(crate) mode: String,
    pub(crate) mode_bits: u32,
    pub(crate) nlink: u64,
    pub(crate) size: u64,
    pub(crate) mtime: SystemTime,
    pub(crate) user: String,
    pub(crate) group: String,
}

/// Runtime choices that influence an otherwise platform-specific layout.
#[derive(Clone, Copy, Debug)]
pub(crate) struct LongFormatLayoutOptions {
    pub(crate) permission_display: PermissionDisplay,
    pub(crate) include_size_unit: bool,
    pub(crate) include_icon: bool,
}

/// A platform-neutral long-format table column.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LongColumn {
    #[cfg_attr(windows, allow(dead_code))]
    UnixSymbolicPermissions,
    #[cfg_attr(windows, allow(dead_code))]
    UnixOctalWithType,
    #[cfg_attr(windows, allow(dead_code))]
    UnixOctal,
    #[cfg_attr(unix, allow(dead_code))]
    Type,
    #[cfg_attr(unix, allow(dead_code))]
    Attributes,
    #[cfg_attr(windows, allow(dead_code))]
    Links,
    #[cfg_attr(windows, allow(dead_code))]
    User,
    #[cfg_attr(windows, allow(dead_code))]
    Group,
    Size,
    Unit,
    Date,
    Icon,
    Name,
}

/// Ordered long-format columns for the active platform.
#[derive(Clone, Debug)]
pub(crate) struct LongFormatLayout {
    pub(crate) columns: Vec<LongColumn>,
}

#[cfg(unix)]
mod unix;

#[cfg(unix)]
pub(crate) use unix::*;

#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub(crate) use windows::*;
