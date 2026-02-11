use std::collections::{HashMap};
use std::fs;
use std::path::PathBuf;
use crate::project::commit::{Commit, FileStatus};
use crate::project::error::{GustError, Result};
use crate::project::paths::{AbsolutePath, CliPath, RootRelativePath};
use crate::project::staging_area::ChangeType;
use super::Root;

impl Root {
    pub(super) fn process_path_list<F>(&mut self, paths: &Vec<PathBuf>, mut apply: F) -> Result<()>
    where F: FnMut(&mut Self, &AbsolutePath) -> Result<()>
    {
        for cli_path in paths.iter().map(|p| CliPath::from(p.as_path())) {
            let absolute_path = AbsolutePath::try_from(cli_path)?;
            if !self.path.is_inside_root(&absolute_path) {
                return Err(GustError::User(format!("Path {:?} is not a root path", absolute_path)));
            }
            if absolute_path.is_dir() {
                for file in self.scan_folder(&absolute_path)? {
                    apply(self, &file)?;
                }
            } else {
                apply(self, &absolute_path)?;
            }
        }
        Ok(())
    }

    pub(super) fn get_changed_files(&self) -> Result<HashMap<RootRelativePath, ChangeType>> {
        let files = self.scan_folder(&AbsolutePath::from_absolute_path(self.path.as_path()))?;
        // Change the get_last_commit_ref name to get_head_tree or something like that
        let commit = Commit::from_commit_ref_option(self.head.get_tree()?, &self.path)?;
        let mut changed_files: HashMap<RootRelativePath, ChangeType> = HashMap::new();

        // Check in the project root directory for changed files
        for file in files {
            let relative_file_path = RootRelativePath::new(&file, &self.path)?;
            if let Some(c) = commit.as_ref() {
                match c.has_file_changed(&relative_file_path, &file)? {
                    FileStatus::Unchanged => continue,
                    status => {
                        let change_type = match status { 
                            FileStatus::Added => ChangeType::Added, 
                            FileStatus::Modified => ChangeType::Modified,
                            _ => unreachable!()
                        };
                        changed_files.insert(relative_file_path, change_type);
                    }
                }
            } else {
                changed_files.insert(relative_file_path, ChangeType::Added);
            }
        }

        if let Some(c) = commit.as_ref() {
            for tracked_file in c.tree_iterator() {
                let absolute_path = &self.path.join_path(tracked_file.0.as_path());
                if !absolute_path.as_path().exists() {
                    changed_files.insert(tracked_file.0.clone(), ChangeType::Removed);
                }
            }
        }

        Ok(changed_files)
    }

    pub fn scan_folder(&self, path: &AbsolutePath) -> Result<Vec<AbsolutePath>> {
        if path.as_path() == &self.path.as_path().join(".gust") {
            return Ok(Vec::new()); // Dont process the root .gust folder
        }

        let entries = fs::read_dir(path.as_path()).unwrap(); // I know the path exists and is a dir
        let mut files = Vec::new();

        for entry in entries {
            let entry_path = AbsolutePath::from_absolute_path(&entry.unwrap().path());
            if entry_path.is_dir() {
                let entry_result = self.scan_folder(&entry_path)?;
                files.extend(entry_result);
            } else {
                files.push(entry_path);
            }
        }
        Ok(files)
    }
}