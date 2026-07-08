//! Unix filesystem metadata interpretation.

use nix::unistd::{Group, User};
use std::ffi::OsStr;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::time::SystemTime;

use crate::structs::NameStyle;
use crate::utils::format;

#[allow(
    clippy::unnecessary_cast,
    reason = "libc::mode_t is u16 on Apple targets"
)]
mod mode_bits {
    pub(super) const FILE_TYPE_MASK: u32 = nix::libc::S_IFMT as u32;
    pub(super) const FIFO: u32 = nix::libc::S_IFIFO as u32;
    pub(super) const CHAR_DEVICE: u32 = nix::libc::S_IFCHR as u32;
    pub(super) const DIRECTORY: u32 = nix::libc::S_IFDIR as u32;
    pub(super) const BLOCK_DEVICE: u32 = nix::libc::S_IFBLK as u32;
    pub(super) const REGULAR: u32 = nix::libc::S_IFREG as u32;
    pub(super) const SYMLINK: u32 = nix::libc::S_IFLNK as u32;
    pub(super) const SOCKET: u32 = nix::libc::S_IFSOCK as u32;
}

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LongFormatFileType {
    Directory,
    Regular,
    Symlink,
    Socket,
    Fifo,
    CharDevice,
    BlockDevice,
    Unknown,
}

impl LongFormatFileType {
    pub(crate) fn as_char(self) -> char {
        match self {
            Self::Directory => 'd',
            Self::Regular => '-',
            Self::Symlink => 'l',
            Self::Socket => 's',
            Self::Fifo => 'p',
            Self::CharDevice => 'c',
            Self::BlockDevice => 'b',
            Self::Unknown => '?',
        }
    }
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

pub(crate) fn metadata_file_type(
    metadata: &fs::Metadata,
) -> LongFormatFileType {
    long_format_file_type(metadata.mode())
}

pub(crate) fn file_details(metadata: &fs::Metadata) -> FileDetails {
    let file_type = metadata_file_type(metadata).as_char().to_string();

    let permissions = metadata.permissions();
    let mode = permissions.mode();
    let rwx_mode = format::mode_to_rwx(mode);

    let nlink = metadata.nlink();
    let size = metadata.size();

    let user = get_username(metadata.uid());
    let group = get_groupname(metadata.gid());

    let mtime = metadata.modified().unwrap();

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
pub fn get_username(uid: u32) -> String {
    match User::from_uid(uid.into()) {
        Ok(Some(user)) => user.name,
        _ => uid.to_string(),
    }
}

/// Look up a group name, falling back to the numeric GID.
pub fn get_groupname(gid: u32) -> String {
    match Group::from_gid(gid.into()) {
        Ok(Some(group)) => group.name,
        _ => gid.to_string(),
    }
}

pub(crate) fn entry_name_is_hidden(name: &OsStr) -> bool {
    name.as_bytes().starts_with(b".")
}

pub(crate) fn sort_key(name: &OsStr) -> Vec<u8> {
    let bytes = name.as_bytes();
    let trimmed = bytes
        .iter()
        .skip_while(|byte| **byte == b'.')
        .copied()
        .collect::<Vec<_>>();
    trimmed
        .into_iter()
        .map(|byte| byte.to_ascii_lowercase())
        .collect()
}

pub(crate) fn is_executable(metadata: &fs::Metadata) -> bool {
    metadata.permissions().mode() & 0o111 != 0
}

pub(crate) fn name_style_for_file_type(
    file_type: LongFormatFileType,
    executable: bool,
) -> NameStyle {
    match file_type {
        LongFormatFileType::Symlink => NameStyle::Symlink,
        LongFormatFileType::Directory => NameStyle::Directory,
        LongFormatFileType::Socket => NameStyle::Socket,
        LongFormatFileType::Fifo => NameStyle::Fifo,
        LongFormatFileType::CharDevice => NameStyle::CharDevice,
        LongFormatFileType::BlockDevice => NameStyle::BlockDevice,
        LongFormatFileType::Regular if executable => NameStyle::Executable,
        LongFormatFileType::Regular | LongFormatFileType::Unknown => {
            NameStyle::Plain
        }
    }
}

pub(crate) fn name_style_by_metadata(metadata: &fs::Metadata) -> NameStyle {
    name_style_for_file_type(
        metadata_file_type(metadata),
        is_executable(metadata),
    )
}
