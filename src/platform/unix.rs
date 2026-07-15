//! Unix filesystem metadata interpretation.

use nix::unistd::{Group, User};
use std::cmp::Ordering;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::platform::{
    EntryClassification, FileDetails, LongColumn, LongFormatFileType,
    LongFormatLayout, LongFormatLayoutOptions,
};
use crate::structs::{AttributeDisplay, NameStyle, Params, PermissionDisplay};
use crate::utils::format;

/// Return whether stdout currently refers to a regular file.
pub(crate) fn stdout_is_regular_file() -> bool {
    let Ok(metadata) = nix::sys::stat::fstat(io::stdout()) else {
        return false;
    };

    let mode = mode_bits::as_u32(metadata.st_mode);
    mode & mode_bits::FILE_TYPE_MASK == mode_bits::REGULAR
}

#[allow(
    clippy::unnecessary_cast,
    reason = "libc::mode_t is u16 on Apple targets"
)]
mod mode_bits {
    pub(super) fn as_u32(mode: nix::libc::mode_t) -> u32 {
        mode as u32
    }

    pub(super) const FILE_TYPE_MASK: u32 = nix::libc::S_IFMT as u32;
    pub(super) const FIFO: u32 = nix::libc::S_IFIFO as u32;
    pub(super) const CHAR_DEVICE: u32 = nix::libc::S_IFCHR as u32;
    pub(super) const DIRECTORY: u32 = nix::libc::S_IFDIR as u32;
    pub(super) const BLOCK_DEVICE: u32 = nix::libc::S_IFBLK as u32;
    pub(super) const REGULAR: u32 = nix::libc::S_IFREG as u32;
    pub(super) const SYMLINK: u32 = nix::libc::S_IFLNK as u32;
    pub(super) const SOCKET: u32 = nix::libc::S_IFSOCK as u32;
}

pub(crate) fn long_format_file_type(mode: u32) -> LongFormatFileType {
    match mode & mode_bits::FILE_TYPE_MASK {
        mode_bits::DIRECTORY => LongFormatFileType::Directory,
        mode_bits::REGULAR => LongFormatFileType::Regular,
        mode_bits::SYMLINK => LongFormatFileType::Symlink,
        mode_bits::SOCKET => LongFormatFileType::Socket,
        mode_bits::FIFO => LongFormatFileType::Fifo,
        mode_bits::CHAR_DEVICE => LongFormatFileType::CharDevice,
        mode_bits::BLOCK_DEVICE => LongFormatFileType::BlockDevice,
        _ => LongFormatFileType::Unknown,
    }
}

pub(crate) fn classify_entry(
    path: &Path,
    metadata: &fs::Metadata,
) -> EntryClassification {
    let file_type = metadata_file_type(path, metadata);
    let is_directory = metadata.is_dir();

    EntryClassification {
        file_type,
        hidden: path
            .file_name()
            .is_some_and(|name| name.as_bytes().starts_with(b".")),
        display_as_directory: if metadata.is_symlink() {
            fs::metadata(path)
                .map(|target| target.is_dir())
                .unwrap_or(false)
        } else {
            is_directory
        },
        group_with_directories: is_directory,
        may_recurse: is_directory && !metadata.is_symlink(),
        may_render_link_target: metadata.is_symlink(),
    }
}

pub(crate) fn entry_name_is_hidden(name: &OsStr) -> bool {
    name.as_bytes().starts_with(b".")
}

pub(crate) fn metadata_file_type(
    _path: &Path,
    metadata: &fs::Metadata,
) -> LongFormatFileType {
    long_format_file_type(metadata.mode())
}

pub(crate) fn file_details(
    _path: &Path,
    metadata: &fs::Metadata,
    classification: EntryClassification,
    _attribute_display: AttributeDisplay,
) -> FileDetails {
    let file_type = classification.file_type.as_char().to_string();

    let permissions = metadata.permissions();
    let mode = permissions.mode();
    let rwx_mode = format::mode_to_rwx(mode);

    let nlink = metadata.nlink();
    let size = metadata.size();

    let user = get_username(metadata.uid());
    let group = get_groupname(metadata.gid());

    let mtime = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);

    FileDetails {
        file_type,
        mode: rwx_mode,
        mode_bits: mode & 0o7777,
        nlink,
        size,
        mtime,
        user,
        group,
    }
}

/// Look up a username, falling back to the numeric UID.
pub(crate) fn get_username(uid: u32) -> String {
    match User::from_uid(uid.into()) {
        Ok(Some(user)) => user.name,
        _ => uid.to_string(),
    }
}

/// Look up a group name, falling back to the numeric GID.
pub(crate) fn get_groupname(gid: u32) -> String {
    match Group::from_gid(gid.into()) {
        Ok(Some(group)) => group.name,
        _ => gid.to_string(),
    }
}

pub(crate) fn compare_entry_names(left: &OsStr, right: &OsStr) -> Ordering {
    fn sort_key(name: &OsStr) -> Vec<u8> {
        name.as_bytes()
            .iter()
            .skip_while(|byte| **byte == b'.')
            .map(|byte| byte.to_ascii_lowercase())
            .collect()
    }

    sort_key(left).cmp(&sort_key(right))
}

pub(crate) fn is_executable(_path: &Path, metadata: &fs::Metadata) -> bool {
    metadata.permissions().mode() & 0o111 != 0
}

pub(crate) fn name_style(
    path: &Path,
    metadata: &fs::Metadata,
    classification: EntryClassification,
) -> NameStyle {
    super::name_style_for_file_type(
        classification.file_type,
        is_executable(path, metadata),
    )
}

pub(crate) fn synthetic_dot_entries(
    params: &Params,
) -> &'static [&'static str] {
    if !params.almost_all && params.show_all {
        &[".", ".."]
    } else {
        &[]
    }
}

pub(crate) fn validate_params(_params: &Params) -> io::Result<()> {
    Ok(())
}

pub(crate) fn default_config_path() -> Option<PathBuf> {
    let mut path = dirs_next::home_dir()?;
    path.push(".config/lsplus/config.toml");
    Some(path)
}

pub(crate) fn normalize_path(path: PathBuf) -> PathBuf {
    path
}

pub(crate) fn long_format_layout(
    options: &LongFormatLayoutOptions,
) -> LongFormatLayout {
    let mut columns = Vec::with_capacity(10);

    match options.permission_display {
        PermissionDisplay::Symbolic => {
            columns.push(LongColumn::UnixSymbolicPermissions);
        }
        PermissionDisplay::Octal => {
            columns.push(LongColumn::UnixOctalWithType);
        }
        PermissionDisplay::Both => {
            columns.push(LongColumn::UnixSymbolicPermissions);
            columns.push(LongColumn::UnixOctal);
        }
        PermissionDisplay::None => {}
    }

    columns.extend([
        LongColumn::Links,
        LongColumn::User,
        LongColumn::Group,
        LongColumn::Size,
    ]);
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
