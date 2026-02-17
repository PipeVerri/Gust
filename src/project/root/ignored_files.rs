use std::fs;
use std::path::PathBuf;
use crate::project::paths::AbsolutePath;
use super::{Root, RootPath};
use crate::project::error::{GustError, Result};

pub enum IgnoredFile {
    Fixed(AbsolutePath),
    Prefixed(PathBuf)
}

impl Root {
    pub(super) fn read_ignored(root_path: &RootPath) -> Result<Vec<IgnoredFile>> {
        let ignore_path = root_path.join(".gustignore");
        if !ignore_path.as_path().exists() { return Ok(vec![]); }
        let file = fs::read_to_string(ignore_path.as_path())?;

        let mut parsed: Vec<IgnoredFile> = vec![];
        for line in file.lines() {
            if line.starts_with('#') { continue; }
            if line.starts_with("/") {
                if let Ok(absolute_path) = root_path.unsafe_join(line.strip_prefix('/').unwrap()) {
                    parsed.push(IgnoredFile::Fixed(absolute_path));
                }
            } else {
                parsed.push(IgnoredFile::Prefixed(PathBuf::from(line)));
            }
        }
        Ok(parsed)
    }

    pub(super) fn is_path_ignored(&self, path: &AbsolutePath) -> Result<bool> {
        for ignored in &self.ignored_files {
            let check = match ignored {
                IgnoredFile::Fixed(relative_path) => {
                    if path == relative_path {
                        true
                    } else if path.as_path().starts_with(relative_path.as_path()) {
                        return Err(GustError::User(format!("Path {} is inside a directory that is ignored", path.as_path().display())))
                    } else {
                        false
                    }
                },
                IgnoredFile::Prefixed(prefix) => {
                    path.as_path().ends_with(prefix.as_path())
                }
            };
            if check { return Ok(true); }
        }
        Ok(false)
    }
}