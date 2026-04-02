use std::collections::HashMap;
use std::fs;
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
    gitignore_matcher: Gitignore,
    git_exclude_matcher: Gitignore,
    git_global_matcher: Gitignore,
}

struct GitPaths {
    root: PathBuf,
    common_dir: PathBuf,
}

impl GitignoreMatcher {
    fn for_directory(directory: &Path) -> Option<Self> {
        let git_paths = find_git_paths(directory)?;
        let ignore_files = collect_gitignore_files(&git_paths.root, directory);

        let mut builder = GitignoreBuilder::new(&git_paths.root);
        for ignore_file in ignore_files {
            let _ = builder.add(ignore_file);
        }
        let gitignore_matcher = builder.build().ok()?;

        let git_exclude_matcher =
            build_git_exclude_matcher(&git_paths.root, &git_paths.common_dir)?;
        let git_global_matcher = build_git_global_matcher(&git_paths.root)?;

        Some(Self {
            root: git_paths.root,
            gitignore_matcher,
            git_exclude_matcher,
            git_global_matcher,
        })
    }

    fn is_ignored(&self, path: &Path, is_dir: bool) -> bool {
        let Ok(relative_path) = path.strip_prefix(&self.root) else {
            return false;
        };

        for matched in [
            self.gitignore_matcher
                .matched_path_or_any_parents(relative_path, is_dir),
            self.git_exclude_matcher.matched(relative_path, is_dir),
            self.git_global_matcher.matched(relative_path, is_dir),
        ] {
            match matched {
                Match::Ignore(_) => return true,
                Match::Whitelist(_) => return false,
                Match::None => {}
            }
        }

        false
    }
}

fn matcher_directory(path: &Path, is_dir: bool) -> &Path {
    if is_dir {
        path
    } else {
        path.parent().unwrap_or_else(|| Path::new("."))
    }
}

fn find_git_paths(start: &Path) -> Option<GitPaths> {
    let mut current = Some(start);

    while let Some(path) = current {
        let dot_git = path.join(".git");

        if dot_git.is_dir() {
            return Some(GitPaths {
                root: path.to_path_buf(),
                common_dir: dot_git,
            });
        }

        if dot_git.is_file() {
            let git_dir = parse_gitdir_file(&dot_git)?;
            let common_dir =
                parse_commondir(&git_dir).unwrap_or_else(|| git_dir.clone());

            return Some(GitPaths {
                root: path.to_path_buf(),
                common_dir,
            });
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

fn build_git_exclude_matcher(
    root: &Path,
    common_dir: &Path,
) -> Option<Gitignore> {
    let mut builder = GitignoreBuilder::new(root);
    let exclude_path = common_dir.join("info").join("exclude");
    if exclude_path.is_file() {
        let _ = builder.add(exclude_path);
    }

    builder.build().ok()
}

fn build_git_global_matcher(root: &Path) -> Option<Gitignore> {
    let (matcher, _err) = GitignoreBuilder::new(root).build_global();
    Some(matcher)
}

fn parse_gitdir_file(dot_git: &Path) -> Option<PathBuf> {
    let contents = fs::read_to_string(dot_git).ok()?;
    let value = contents.strip_prefix("gitdir:")?.trim();
    let git_dir = PathBuf::from(value);

    if git_dir.is_absolute() {
        Some(normalize_path(git_dir))
    } else {
        Some(normalize_path(dot_git.parent()?.join(git_dir)))
    }
}

fn parse_commondir(git_dir: &Path) -> Option<PathBuf> {
    let contents = fs::read_to_string(git_dir.join("commondir")).ok()?;
    let common_dir = PathBuf::from(contents.trim());

    if common_dir.is_absolute() {
        Some(normalize_path(common_dir))
    } else {
        Some(normalize_path(git_dir.join(common_dir)))
    }
}

fn normalize_path(path: PathBuf) -> PathBuf {
    fs::canonicalize(&path).unwrap_or(path)
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

    #[test]
    fn test_find_git_paths_resolves_linked_worktree_common_dir() {
        let temp_dir = tempdir().unwrap();
        let repo_root = temp_dir.path().join("repo");
        let common_dir = temp_dir.path().join("common");
        let git_dir = common_dir.join("worktrees").join("repo");

        std::fs::create_dir_all(&repo_root).unwrap();
        std::fs::create_dir_all(&git_dir).unwrap();
        std::fs::write(
            repo_root.join(".git"),
            format!("gitdir: {}\n", git_dir.display()),
        )
        .unwrap();
        std::fs::write(git_dir.join("commondir"), "../../\n").unwrap();

        let git_paths = find_git_paths(&repo_root).unwrap();

        assert_eq!(git_paths.root, repo_root);
        assert_eq!(git_paths.common_dir, common_dir);
    }
}
