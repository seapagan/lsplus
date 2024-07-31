use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitIgnoreError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Failed to compile regex pattern: {0}")]
    RegexError(#[from] regex::Error),

    #[error("Invalid file path: {0}")]
    InvalidFilePath(PathBuf),
}

#[macro_export]
macro_rules! gitignore {
    () => {
        GitIgnore::new(None)
    };
    ($path:expr) => {
        GitIgnore::new(Some($path))
    };
}

/// Represents a .gitignore file and its patterns
pub struct GitIgnore {
    pub patterns: Option<Vec<(bool, Regex, String)>>,
}

impl GitIgnore {
    pub fn new(path: Option<&str>) -> Result<Self, GitIgnoreError> {
        let patterns = GitIgnore::find_gitignore(path)
            .map(|gitignore_path| {
                GitIgnore::read_gitignore_patterns(&gitignore_path)
            })
            .transpose()?
            .map(|patterns| {
                patterns
                    .into_iter()
                    .map(|pattern| {
                        let (negated, pattern_str) = if let Some(stripped) =
                            pattern.strip_prefix('!')
                        {
                            (true, stripped.to_string())
                        } else {
                            (false, pattern)
                        };

                        let regex_pattern = if let Some(stripped_pattern) =
                            pattern_str.strip_prefix('/')
                        {
                            format!(
                                r"^/{}",
                                regex::escape(stripped_pattern)
                                    .replace(r"\*\*", ".*")
                                    .replace(r"\*", "[^/]*")
                                    .replace(r"\?", ".")
                            )
                        } else if pattern_str.contains('/') {
                            regex::escape(&pattern_str)
                                .replace(r"\*\*", ".*")
                                .replace(r"\*", "[^/]*")
                                .replace(r"\?", ".")
                        } else {
                            format!(
                                r"(^|.*/){}$",
                                regex::escape(&pattern_str)
                                    .replace(r"\*\*", ".*")
                                    .replace(r"\*", "[^/]*")
                                    .replace(r"\?", ".")
                            )
                        };

                        (
                            negated,
                            Regex::new(&regex_pattern).unwrap(),
                            pattern_str,
                        )
                    })
                    .collect()
            });

        Ok(GitIgnore { patterns })
    }

    fn find_gitignore(path: Option<&str>) -> Option<PathBuf> {
        let start_dir = path.map(PathBuf::from).unwrap_or_else(|| {
            std::env::current_dir().expect("Failed to get current directory")
        });
        let mut dir = Some(start_dir.as_path());

        while let Some(parent) = dir {
            let gitignore_path = parent.join(".gitignore");
            if gitignore_path.exists() {
                return Some(gitignore_path);
            }
            if parent.join(".git").exists() {
                break;
            }
            dir = parent.parent();
        }
        None
    }

    fn read_gitignore_patterns(
        gitignore_path: &Path,
    ) -> Result<Vec<String>, GitIgnoreError> {
        let file = File::open(gitignore_path)?;
        let reader = BufReader::new(file);
        let mut patterns = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                patterns.push(trimmed.to_string());
            }
        }
        Ok(patterns)
    }

    pub fn is_file_ignored(
        &self,
        file_path: &str,
    ) -> Result<bool, GitIgnoreError> {
        if let Some(patterns) = &self.patterns {
            let file_path = Path::new(file_path);
            let normalized_path = file_path
                .to_str()
                .ok_or_else(|| {
                    GitIgnoreError::InvalidFilePath(file_path.to_path_buf())
                })?
                .replace('\\', "/");

            let mut is_ignored = false;

            for (negated, regex, _) in patterns {
                if regex.is_match(&normalized_path) {
                    is_ignored = !negated;
                }
            }

            Ok(is_ignored)
        } else {
            Ok(false)
        }
    }
}
