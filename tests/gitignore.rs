use lsplus::utils::gitignore::{
    GitignoreCache, collect_gitignore_files, find_git_paths_parts,
    matcher_ignores_path, parse_commondir, parse_gitdir_file,
};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_gitignore_cache_matches_parent_and_child_rules() {
    let temp_dir = tempdir().unwrap();
    let repo_root = temp_dir.path();
    let nested_dir = repo_root.join("nested");
    let ignored_file = nested_dir.join("ignored.log");
    let kept_file = nested_dir.join("keep.log");

    fs::create_dir(repo_root.join(".git")).unwrap();
    fs::create_dir(&nested_dir).unwrap();
    fs::write(repo_root.join(".gitignore"), "*.log\n").unwrap();
    fs::write(nested_dir.join(".gitignore"), "!keep.log\n").unwrap();
    fs::write(&ignored_file, "ignored").unwrap();
    fs::write(&kept_file, "kept").unwrap();

    let mut cache = GitignoreCache::default();

    assert!(cache.is_ignored(&ignored_file, false));
    assert!(!cache.is_ignored(&kept_file, false));
}

#[test]
fn test_gitignore_cache_returns_false_outside_worktree() {
    let temp_dir = tempdir().unwrap();
    let plain_file = temp_dir.path().join("plain.txt");
    fs::write(&plain_file, "plain").unwrap();

    let mut cache = GitignoreCache::default();

    assert!(!cache.is_ignored(&plain_file, false));
}

#[test]
fn test_gitignore_cache_ignores_invalid_gitignore_files() {
    let temp_dir = tempdir().unwrap();
    let repo_root = temp_dir.path();
    let broken_file = repo_root.join("broken.log");

    fs::create_dir(repo_root.join(".git")).unwrap();
    fs::write(repo_root.join(".gitignore"), "[\n").unwrap();
    fs::write(&broken_file, "broken").unwrap();

    let mut cache = GitignoreCache::default();

    assert!(!cache.is_ignored(&broken_file, false));
}

#[test]
fn test_find_git_paths_resolves_linked_worktree_common_dir() {
    let temp_dir = tempdir().unwrap();
    let repo_root = temp_dir.path().join("repo");
    let common_dir = temp_dir.path().join("common");
    let git_dir = common_dir.join("worktrees").join("repo");

    fs::create_dir_all(&repo_root).unwrap();
    fs::create_dir_all(&git_dir).unwrap();
    fs::write(
        repo_root.join(".git"),
        format!("gitdir: {}\n", git_dir.display()),
    )
    .unwrap();
    fs::write(git_dir.join("commondir"), "../../\n").unwrap();

    let (root, resolved_common_dir) =
        find_git_paths_parts(&repo_root).unwrap();

    assert_eq!(root, repo_root);
    assert_eq!(resolved_common_dir, common_dir);
}

#[test]
fn test_collect_gitignore_files_handles_outside_root_and_parentless_paths() {
    let temp_dir = tempdir().unwrap();
    let root = temp_dir.path().join("repo");
    let nested = root.join("nested");

    fs::create_dir_all(&nested).unwrap();
    fs::write(root.join(".gitignore"), "root\n").unwrap();
    fs::write(nested.join(".gitignore"), "nested\n").unwrap();

    let files = collect_gitignore_files(&root, &nested);
    assert_eq!(
        files,
        vec![root.join(".gitignore"), nested.join(".gitignore")]
    );

    assert!(collect_gitignore_files(&root, temp_dir.path()).is_empty());
    assert!(
        collect_gitignore_files(
            temp_dir.path(),
            std::path::Path::new("relative")
        )
        .is_empty()
    );
}

#[test]
fn test_parse_gitdir_file_handles_relative_paths() {
    let temp_dir = tempdir().unwrap();
    let repo_root = temp_dir.path().join("repo");
    let git_dir = temp_dir.path().join("gitdir");

    fs::create_dir_all(&repo_root).unwrap();
    fs::create_dir_all(&git_dir).unwrap();
    fs::write(repo_root.join(".git"), "gitdir: ../gitdir\n").unwrap();

    assert_eq!(parse_gitdir_file(&repo_root.join(".git")), Some(git_dir));
}

#[test]
fn test_parse_gitdir_file_returns_none_for_missing_and_malformed_files() {
    let temp_dir = tempdir().unwrap();
    let missing = temp_dir.path().join("missing.git");
    let malformed = temp_dir.path().join(".git");

    fs::write(&malformed, "not-a-gitdir-line\n").unwrap();

    assert_eq!(parse_gitdir_file(&missing), None);
    assert_eq!(parse_gitdir_file(&malformed), None);
}

#[test]
fn test_parse_commondir_handles_absolute_paths() {
    let temp_dir = tempdir().unwrap();
    let git_dir = temp_dir.path().join("gitdir");
    let common_dir = temp_dir.path().join("common");

    fs::create_dir_all(&git_dir).unwrap();
    fs::create_dir_all(&common_dir).unwrap();
    fs::write(
        git_dir.join("commondir"),
        format!("{}\n", common_dir.display()),
    )
    .unwrap();

    assert_eq!(parse_commondir(&git_dir), Some(common_dir));
}

#[test]
fn test_parse_commondir_returns_none_when_file_is_missing() {
    let temp_dir = tempdir().unwrap();
    let git_dir = temp_dir.path().join("gitdir");

    fs::create_dir_all(&git_dir).unwrap();

    assert_eq!(parse_commondir(&git_dir), None);
}

#[test]
fn test_matcher_ignores_path_returns_false_outside_matcher_root() {
    let temp_dir = tempdir().unwrap();
    let repo_root = temp_dir.path().join("repo");
    let outside = temp_dir.path().join("outside.log");

    fs::create_dir_all(&repo_root).unwrap();
    fs::create_dir(repo_root.join(".git")).unwrap();
    fs::write(repo_root.join(".gitignore"), "*.log\n").unwrap();
    fs::write(&outside, "outside").unwrap();

    assert_eq!(
        matcher_ignores_path(&repo_root, &outside, false),
        Some(false)
    );
}

#[test]
fn test_helper_seams_return_none_when_no_git_metadata_exists() {
    let temp_dir = tempdir().unwrap();
    let relative_file = std::path::Path::new("plain.txt");
    let mut cache = GitignoreCache::default();

    assert_eq!(find_git_paths_parts(temp_dir.path()), None);
    assert_eq!(
        matcher_ignores_path(temp_dir.path(), temp_dir.path(), true),
        None
    );
    assert!(!cache.is_ignored(relative_file, false));
}

#[test]
fn test_find_git_paths_uses_git_dir_when_commondir_is_missing() {
    let temp_dir = tempdir().unwrap();
    let repo_root = temp_dir.path().join("repo");
    let git_dir = temp_dir.path().join("gitdir");

    fs::create_dir_all(&repo_root).unwrap();
    fs::create_dir_all(&git_dir).unwrap();
    fs::write(
        repo_root.join(".git"),
        format!("gitdir: {}\n", git_dir.display()),
    )
    .unwrap();

    assert_eq!(find_git_paths_parts(&repo_root), Some((repo_root, git_dir)));
}

#[test]
fn test_find_git_paths_returns_none_for_malformed_gitdir_file() {
    let temp_dir = tempdir().unwrap();
    let repo_root = temp_dir.path().join("repo");

    fs::create_dir_all(&repo_root).unwrap();
    fs::write(repo_root.join(".git"), "not-a-gitdir-line\n").unwrap();

    assert_eq!(find_git_paths_parts(&repo_root), None);
}
