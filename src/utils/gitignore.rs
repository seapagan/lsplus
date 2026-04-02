use std::collections::HashMap;
use std::path::{Path, PathBuf};

use ignore::Match;
use ignore::gitignore::{Gitignore, GitignoreBuilder};

#[derive(Default)]
pub struct GitignoreCache {
    matchers: HashMap<PathBuf, Option<GitignoreMatcher>>,
}

impl GitignoreCache {
    pub fn is_ignored(&mut self, path: &Path, is_dir: bool) -> bool {
        let key = matcher_directory(path, is_dir).to_path_buf();
        let matcher = self
            .matchers
            .entry(key.clone())
            .or_insert_with(|| GitignoreMatcher::for_directory(&key));

        matcher
            .as_ref()
            .is_some_and(|gitignore| gitignore.is_ignored(path, is_dir))
    }
}

struct GitignoreMatcher {
    root: PathBuf,
    matcher: Gitignore,
}

impl GitignoreMatcher {
    fn for_directory(directory: &Path) -> Option<Self> {
        let root = find_git_root(directory)?;
        let ignore_files = collect_gitignore_files(&root, directory);

        if ignore_files.is_empty() {
            return None;
        }

        let mut builder = GitignoreBuilder::new(&root);
        for ignore_file in ignore_files {
            let _ = builder.add(ignore_file);
        }

        let matcher = builder.build().ok()?;

        Some(Self { root, matcher })
    }

    fn is_ignored(&self, path: &Path, is_dir: bool) -> bool {
        let Ok(relative_path) = path.strip_prefix(&self.root) else {
            return false;
        };

        matches!(
            self.matcher
                .matched_path_or_any_parents(relative_path, is_dir),
            Match::Ignore(_)
        )
    }
}

fn matcher_directory(path: &Path, is_dir: bool) -> &Path {
    if is_dir {
        path
    } else {
        path.parent().unwrap_or_else(|| Path::new("."))
    }
}

fn find_git_root(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);

    while let Some(path) = current {
        if path.join(".git").exists() {
            return Some(path.to_path_buf());
        }

        current = path.parent();
    }

    None
}

fn collect_gitignore_files(root: &Path, directory: &Path) -> Vec<PathBuf> {
    if !directory.starts_with(root) {
        return Vec::new();
    }

    let mut directories = Vec::new();
    let mut current = directory;

    loop {
        directories.push(current.to_path_buf());
        if current == root {
            break;
        }

        let Some(parent) = current.parent() else {
            return Vec::new();
        };
        current = parent;
    }

    directories.reverse();

    directories
        .into_iter()
        .map(|path| path.join(".gitignore"))
        .filter(|path| path.is_file())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_gitignore_cache_matches_parent_and_child_rules() {
        let temp_dir = tempdir().unwrap();
        let repo_root = temp_dir.path();
        let nested_dir = repo_root.join("nested");
        let ignored_file = nested_dir.join("ignored.log");
        let kept_file = nested_dir.join("keep.log");

        std::fs::create_dir(repo_root.join(".git")).unwrap();
        std::fs::create_dir(&nested_dir).unwrap();
        std::fs::write(repo_root.join(".gitignore"), "*.log\n").unwrap();
        std::fs::write(nested_dir.join(".gitignore"), "!keep.log\n").unwrap();
        std::fs::write(&ignored_file, "ignored").unwrap();
        std::fs::write(&kept_file, "kept").unwrap();

        let mut cache = GitignoreCache::default();

        assert!(cache.is_ignored(&ignored_file, false));
        assert!(!cache.is_ignored(&kept_file, false));
    }

    #[test]
    fn test_gitignore_cache_returns_false_outside_worktree() {
        let temp_dir = tempdir().unwrap();
        let plain_file = temp_dir.path().join("plain.txt");
        std::fs::write(&plain_file, "plain").unwrap();

        let mut cache = GitignoreCache::default();

        assert!(!cache.is_ignored(&plain_file, false));
    }
}
