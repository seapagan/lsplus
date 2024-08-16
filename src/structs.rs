use super::utils::icons::Icon;
use std::path::PathBuf;
use std::time::SystemTime;

pub struct Params {
    pub show_all: bool,
    pub append_slash: bool,
    pub dirs_first: bool,
    pub almost_all: bool,
    pub long_format: bool,
    pub human_readable: bool,
    pub no_icons: bool,
    pub fuzzy_time: bool,
}
#[derive(Debug)]
pub struct FileInfo {
    pub file_type: String,
    pub mode: String,
    pub nlink: u64,
    pub user: String,
    pub group: String,
    pub size: u64,
    pub mtime: SystemTime,
    pub item_icon: Option<Icon>,
    pub display_name: String,
    pub full_path: PathBuf,
}
