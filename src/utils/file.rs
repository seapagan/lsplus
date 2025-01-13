use nix::unistd::{Group, User};
use std::fs;
use std::io;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;
use std::time::SystemTime;

use inline_colorization::*;

use crate::structs::FileInfo;
use crate::utils;
use crate::utils::format;
use crate::Params;

pub fn get_file_details(
    metadata: &fs::Metadata,
) -> (String, String, u64, u64, SystemTime, String, String, bool) {
    let file_type = if metadata.is_dir() {
        "d"
    } else if metadata.is_file() {
        "-"
    } else if metadata.is_symlink() {
        "l"
    } else {
        "?"
    }
    .to_string();

    let permissions = metadata.permissions();
    let mode = permissions.mode();
    let rwx_mode = format::mode_to_rwx(mode);

    let nlink = metadata.nlink();
    let size = metadata.size();

    let user = get_username(metadata.uid());
    let group = get_groupname(metadata.gid());

    let mtime = metadata.modified().unwrap();

    #[cfg(unix)]
    let executable = metadata.permissions().mode() & 0o111 != 0;

    // for now just return false under windows
    #[cfg(windows)]
    let executable = false;

    (
        file_type, rwx_mode, nlink, size, mtime, user, group, executable,
    )
}

pub fn collect_file_names(
    path: &Path,
    params: &Params,
) -> io::Result<Vec<String>> {
    let mut file_names = Vec::new();

    let path_metadata = fs::symlink_metadata(path)?;

    if !path_metadata.is_dir() {
        // If it's a file or symlink, add it directly to the file_names vector
        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        file_names.push(file_name);
    } else {
        // If it's a directory, read its entries
        let mut entries: Vec<fs::DirEntry> = fs::read_dir(path)?
            .filter_map(Result::ok)
            .filter(|entry| {
                if params.show_all || params.almost_all {
                    true
                } else {
                    entry
                        .file_name()
                        .to_str()
                        .map(|s| !s.starts_with('.'))
                        .unwrap_or(false)
                }
            })
            .collect();

        // Sort entries alphabetically, ignoring leading dots
        entries.sort_by(|a, b| {
            let a_name = a
                .file_name()
                .to_str()
                .unwrap()
                .trim_start_matches('.')
                .to_lowercase();
            let b_name = b
                .file_name()
                .to_str()
                .unwrap()
                .trim_start_matches('.')
                .to_lowercase();
            a_name.cmp(&b_name)
        });

        // Separate directories and files if dirs_first is true
        if params.dirs_first {
            let (dirs, files): (Vec<_>, Vec<_>) =
                entries.into_iter().partition(|entry| {
                    entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                });

            entries = dirs.into_iter().chain(files).collect();
        }

        if !params.almost_all && params.show_all {
            file_names.push(".".to_string());
            file_names.push("..".to_string());
        }

        for entry in entries {
            file_names.push(entry.file_name().to_string_lossy().into_owned())
        }
    }
    Ok(file_names)
}

pub fn get_username(uid: u32) -> String {
    match User::from_uid(uid.into()) {
        Ok(Some(user)) => user.name,
        _ => uid.to_string(),
    }
}

pub fn get_groupname(gid: u32) -> String {
    match Group::from_gid(gid.into()) {
        Ok(Some(group)) => group.name,
        _ => gid.to_string(),
    }
}

pub fn collect_file_info(
    path: &Path,
    params: &Params,
) -> io::Result<Vec<FileInfo>> {
    let mut file_info = Vec::new();
    let metadata = fs::symlink_metadata(path)?;

    if metadata.is_dir() {
        let file_names = utils::file::collect_file_names(path, params)?;

        for file_name in file_names {
            let full_path = path.join(&file_name);
            if let Ok(info) = create_file_info(&full_path, params) {
                file_info.push(info);
            }
        }
    } else if let Ok(info) = create_file_info(path, params) {
        file_info.push(info);
    }
    Ok(file_info)
}

pub fn create_file_info(path: &Path, params: &Params) -> io::Result<FileInfo> {
    let metadata = fs::symlink_metadata(path)?;
    let item_icon = if params.no_icons {
        None
    } else {
        Some(utils::icons::get_item_icon(
            &metadata,
            &path.to_string_lossy(),
        ))
    };
    let (file_type, mode, nlink, size, mtime, user, group, executable) =
        utils::file::get_file_details(&metadata);

    let mut file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned());

    if file_name.starts_with("./") {
        file_name = file_name.replacen("./", "", 1);
    }

    if params.append_slash && metadata.is_dir() {
        file_name.push('/');
    }

    let display_name = if metadata.is_symlink() {
        match fs::read_link(path) {
            Ok(target) => {
                let target_path = if target.is_relative() {
                    path.parent().unwrap_or(Path::new("")).join(target)
                } else {
                    target
                };
                if params.long_format {
                    if target_path.exists() {
                        format!(
                            "{color_cyan}{} -> {}",
                            file_name,
                            target_path.display()
                        )
                    } else {
                        format!(
                            "{color_cyan}{} -> {} {color_red}[Broken Link]",
                            file_name,
                            target_path.display()
                        )
                    }
                } else {
                    format!(
                        "{color_cyan}{}{}",
                        file_name,
                        if params.append_slash { "*" } else { "" }
                    )
                }
            }
            Err(_) => {
                if params.long_format {
                    format!("{color_red}{} -> (unreadable)", file_name)
                } else {
                    format!(
                        "{color_cyan}{}{}",
                        file_name,
                        if params.append_slash { "*" } else { "" }
                    )
                }
            }
        }
    } else if metadata.is_dir() {
        format!("{color_blue}{}", file_name)
    } else if executable {
        format!("{style_bold}{color_green}{}", file_name)
    } else {
        // Regular files must have explicit color formatting (even if just reset)
        // to ensure consistent ANSI escape sequence handling across all file types.
        // This maintains proper alignment in table display format.
        format!("{color_reset}{}", file_name)
    };

    Ok(FileInfo {
        file_type,
        mode,
        nlink,
        user,
        group,
        size,
        mtime,
        item_icon,
        display_name,
        full_path: path.to_path_buf(),
    })
}

pub fn check_display_name(info: &FileInfo) -> String {
    match &info.full_path.to_string_lossy() {
        p if p.ends_with("/.") => format!("{color_blue}."),
        p if p.ends_with("/..") => format!("{color_blue}.."),
        _ => info.display_name.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_check_display_name() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        File::create(&file_path).unwrap();

        let info = FileInfo {
            file_type: String::from("regular file"),
            mode: String::from("-rw-r--r--"),
            nlink: 1,
            user: String::from("user"),
            group: String::from("group"),
            size: 0,
            mtime: SystemTime::now(),
            item_icon: None,
            display_name: String::from("test.txt"),
            full_path: file_path,
        };

        let result = check_display_name(&info);
        assert_eq!(result, "test.txt");

        // Test with directory
        let dir_path = temp_dir.path().join("testdir");
        fs::create_dir(&dir_path).unwrap();
        
        let info = FileInfo {
            file_type: String::from("directory"),
            mode: String::from("drwxr-xr-x"),
            nlink: 2,
            user: String::from("user"),
            group: String::from("group"),
            size: 0,
            mtime: SystemTime::now(),
            item_icon: None,
            display_name: String::from("testdir"),
            full_path: dir_path,
        };

        let result = check_display_name(&info);
        assert_eq!(result, "testdir");

        // Test with . directory
        let dot_info = FileInfo {
            file_type: String::from("directory"),
            mode: String::from("drwxr-xr-x"),
            nlink: 2,
            user: String::from("user"),
            group: String::from("group"),
            size: 0,
            mtime: SystemTime::now(),
            item_icon: None,
            display_name: String::from("."),
            full_path: temp_dir.path().join("."),
        };

        let result = check_display_name(&dot_info);
        assert_eq!(result, format!("{color_blue}."));
    }

    #[test]
    fn test_collect_file_info() -> io::Result<()> {
        let temp_dir = tempdir()?;
        
        // Create test files and directories
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        let dir1 = temp_dir.path().join("dir1");
        let hidden = temp_dir.path().join(".hidden");
        
        File::create(&file1)?;
        File::create(&file2)?;
        File::create(&hidden)?;
        fs::create_dir(&dir1)?;

        // Test default params
        let params = Params::default();
        let info = collect_file_info(temp_dir.path(), &params)?;
        assert_eq!(info.len(), 3); // 2 files + 1 dir, hidden file not included
        
        // Test show_all
        let params = Params {
            show_all: true,
            ..Default::default()
        };
        let info = collect_file_info(temp_dir.path(), &params)?;
        assert_eq!(info.len(), 6); // Including hidden file + . and ..

        // Test dirs_first
        let params = Params {
            dirs_first: true,
            ..Default::default()
        };
        let info = collect_file_info(temp_dir.path(), &params)?;
        // Find all directories in the list
        let dir_count = info.iter().filter(|f| f.file_type == "d").count();
        assert!(dir_count > 0, "Should have at least one directory");
        // Check that all directories come before files
        let first_file_idx = info.iter().position(|f| f.file_type != "d");
        if let Some(idx) = first_file_idx {
            assert!(info[..idx].iter().all(|f| f.file_type == "d"), 
                "All items before first file should be directories");
            assert!(info[idx..].iter().all(|f| f.file_type != "d"), 
                "All items after first file should not be directories");
        }

        Ok(())
    }

    #[test]
    fn test_get_file_info() -> io::Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test content")?;

        let params = Params::default();
        let info = create_file_info(&file_path, &params)?;

        assert_eq!(info.display_name, format!("{color_reset}test.txt"));
        assert!(!info.full_path.is_dir());
        assert_eq!(info.size, 12); // "test content" is 12 bytes
        assert!(info.item_icon.is_some());

        Ok(())
    }

    #[test]
    fn test_sort_file_info() {
        let mut files = vec![
            FileInfo {
                file_type: String::from("regular file"),
                mode: String::from("-rw-r--r--"),
                nlink: 1,
                user: String::from("user"),
                group: String::from("group"),
                size: 0,
                mtime: SystemTime::now(),
                item_icon: None,
                display_name: String::from("b.txt"),
                full_path: PathBuf::from("b.txt"),
            },
            FileInfo {
                file_type: String::from("regular file"),
                mode: String::from("-rw-r--r--"),
                nlink: 1,
                user: String::from("user"),
                group: String::from("group"),
                size: 0,
                mtime: SystemTime::now(),
                item_icon: None,
                display_name: String::from("a.txt"),
                full_path: PathBuf::from("a.txt"),
            },
            FileInfo {
                file_type: String::from("directory"),
                mode: String::from("drwxr-xr-x"),
                nlink: 2,
                user: String::from("user"),
                group: String::from("group"),
                size: 0,
                mtime: SystemTime::now(),
                item_icon: None,
                display_name: String::from("dir"),
                full_path: PathBuf::from("dir"),
            },
        ];

        // Test normal sort
        files.sort_by(|a, b| a.display_name.cmp(&b.display_name));
        assert_eq!(files[0].display_name, "a.txt");
        assert_eq!(files[1].display_name, "b.txt");
        assert_eq!(files[2].display_name, "dir");

        // Test dirs_first
        files.sort_by(|a, b| {
            if a.file_type == "directory" && b.file_type != "directory" {
                std::cmp::Ordering::Less
            } else if a.file_type != "directory" && b.file_type == "directory" {
                std::cmp::Ordering::Greater
            } else {
                a.display_name.cmp(&b.display_name)
            }
        });
        assert_eq!(files[0].display_name, "dir");
        assert_eq!(files[1].display_name, "a.txt");
        assert_eq!(files[2].display_name, "b.txt");
    }

    #[test]
    fn test_get_file_details() -> io::Result<()> {
        // Create test files with different types
        let dir_path = Path::new("test_dir");
        let file_path = Path::new("test_file");
        let symlink_path = Path::new("test_symlink");
        let special_path = Path::new("test_special");

        fs::create_dir(dir_path)?;
        File::create(file_path)?;
        std::os::unix::fs::symlink(file_path, symlink_path)?;

        // Test directory
        let metadata = fs::metadata(dir_path)?;
        let (file_type, _, _, _, _, _, _, _) = get_file_details(&metadata);
        assert_eq!(file_type, "d");

        // Test regular file
        let metadata = fs::metadata(file_path)?;
        let (file_type, _, _, _, _, _, _, executable) = get_file_details(&metadata);
        assert_eq!(file_type, "-");
        assert!(!executable);

        // Test symlink
        let metadata = fs::symlink_metadata(symlink_path)?;
        let (file_type, _, _, _, _, _, _, _) = get_file_details(&metadata);
        assert_eq!(file_type, "l");

        // Test executable file
        std::fs::set_permissions(file_path, fs::Permissions::from_mode(0o755))?;
        let metadata = fs::metadata(file_path)?;
        let (_, _, _, _, _, _, _, executable) = get_file_details(&metadata);
        assert!(executable);

        // Cleanup
        fs::remove_dir(dir_path)?;
        fs::remove_file(file_path)?;
        fs::remove_file(symlink_path)?;
        Ok(())
    }

    #[test]
    fn test_get_username_groupname() {
        // Test existing user
        let root_uid = 0;
        let username = get_username(root_uid);
        assert!(username == "root" || username == "0");

        // Test non-existent user
        let nonexistent_uid = u32::MAX;
        let username = get_username(nonexistent_uid);
        assert_eq!(username, nonexistent_uid.to_string());

        // Test existing group
        let root_gid = 0;
        let groupname = get_groupname(root_gid);
        assert!(groupname == "root" || groupname == "0");

        // Test non-existent group
        let nonexistent_gid = u32::MAX;
        let groupname = get_groupname(nonexistent_gid);
        assert_eq!(groupname, nonexistent_gid.to_string());
    }

    #[test]
    fn test_create_file_info_symlinks() -> io::Result<()> {
        // Create temporary test directory
        let temp_dir = tempfile::tempdir()?;
        let file_path = temp_dir.path().join("test_file");
        let valid_symlink = temp_dir.path().join("valid_symlink");
        let broken_symlink = temp_dir.path().join("broken_symlink");
        let unreadable_symlink = temp_dir.path().join("unreadable_symlink");

        // Create the test file and symlinks
        File::create(&file_path)?;
        std::os::unix::fs::symlink(&file_path, &valid_symlink)?;
        std::os::unix::fs::symlink("nonexistent", &broken_symlink)?;
        std::os::unix::fs::symlink("/dev/null", &unreadable_symlink)?;

        // Test valid symlink with long format
        let mut params = Params::default();
        params.long_format = true;
        let info = create_file_info(&valid_symlink, &params)?;
        assert!(info.display_name.contains("->"));
        assert!(!info.display_name.contains("[Broken Link]"));

        // Test broken symlink with long format
        let info = create_file_info(&broken_symlink, &params)?;
        assert!(info.display_name.contains("->"));
        assert!(info.display_name.contains("[Broken Link]"));

        // Test symlink without long format but with append_slash
        params.long_format = false;
        params.append_slash = true;
        let info = create_file_info(&valid_symlink, &params)?;
        assert!(info.display_name.contains("*"));

        Ok(())
    }

    #[test]
    fn test_create_file_info_special_cases() -> io::Result<()> {
        // Create temporary test directory
        let temp_dir = tempfile::tempdir()?;
        let dir_path = temp_dir.path().join("test_dir");
        let file_path = temp_dir.path().join("test_file");
        
        fs::create_dir(&dir_path)?;
        File::create(&file_path)?;

        // Test directory with append_slash
        let mut params = Params::default();
        params.append_slash = true;
        let info = create_file_info(&dir_path, &params)?;
        assert!(info.display_name.ends_with('/'));

        // Test file with ./ prefix
        let info = create_file_info(&file_path, &params)?;
        assert!(!info.display_name.starts_with("./"));

        // Test with no_icons parameter
        params.no_icons = true;
        let info = create_file_info(&file_path, &params)?;
        assert!(info.item_icon.is_none());

        // Test executable file
        std::fs::set_permissions(&file_path, fs::Permissions::from_mode(0o755))?;
        let info = create_file_info(&file_path, &params)?;
        assert!(info.display_name.contains(color_green));

        Ok(())
    }

    #[test]
    fn test_collect_file_names() -> io::Result<()> {
        // Create temporary test directory
        let temp_dir = tempfile::tempdir()?;
        let file1_path = temp_dir.path().join(".hidden_file");
        let file2_path = temp_dir.path().join("visible_file");
        let subdir_path = temp_dir.path().join("subdir");

        fs::create_dir(&subdir_path)?;
        File::create(&file1_path)?;
        File::create(&file2_path)?;

        // Test with show_all = false (default)
        let params = Params::default();
        let files = collect_file_names(temp_dir.path(), &params)?;
        assert!(!files.contains(&".hidden_file".to_string()));
        assert!(files.contains(&"visible_file".to_string()));

        // Test with show_all = true
        let mut params = Params::default();
        params.show_all = true;
        let files = collect_file_names(temp_dir.path(), &params)?;
        assert!(files.contains(&".".to_string()));
        assert!(files.contains(&"..".to_string()));
        assert!(files.contains(&".hidden_file".to_string()));

        // Test with almost_all = true
        let mut params = Params::default();
        params.almost_all = true;
        let files = collect_file_names(temp_dir.path(), &params)?;
        assert!(!files.contains(&".".to_string()));
        assert!(!files.contains(&"..".to_string()));
        assert!(files.contains(&".hidden_file".to_string()));

        // Test with dirs_first = true
        let mut params = Params::default();
        params.dirs_first = true;
        let files = collect_file_names(temp_dir.path(), &params)?;
        let subdir_idx = files.iter().position(|x| x == "subdir").unwrap();
        let file_idx = files.iter().position(|x| x == "visible_file").unwrap();
        assert!(subdir_idx < file_idx);

        // Test with a regular file (not a directory)
        let files = collect_file_names(&file2_path, &params)?;
        assert_eq!(files, vec!["visible_file"]);

        Ok(())
    }

    #[test]
    fn test_create_file_info_edge_cases() -> io::Result<()> {
        // Create temporary test directory
        let temp_dir = tempfile::tempdir()?;
        let file_path = temp_dir.path().join("test_file");
        let symlink_path = temp_dir.path().join("test_symlink");
        
        // Create test file and symlink
        File::create(&file_path)?;
        std::os::unix::fs::symlink("nonexistent", &symlink_path)?;

        // Test broken symlink with long format
        let mut params = Params::default();
        params.long_format = true;
        let info = create_file_info(&symlink_path, &params)?;
        assert!(info.display_name.contains("[Broken Link]"));

        // Test symlink with append_slash but not long_format
        params.long_format = false;
        params.append_slash = true;
        let info = create_file_info(&symlink_path, &params)?;
        assert!(info.display_name.contains("*"));

        Ok(())
    }
}
