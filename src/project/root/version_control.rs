use std::collections::HashSet;
use std::fs;
use crate::project::commit::Commit;
use crate::project::paths::{AbsolutePath, CliPath, RootRelativePath};
use super::{Root, Result, GustError};

impl Root {
    pub fn add(&mut self, paths: Vec<CliPath>) -> Result<()> {
        let changed_files = self.get_changed_files()?;
        for path in paths {
            let absolute_path = AbsolutePath::try_from(path)?; // Checks if it exists
            if !self.path.is_inside_root(&absolute_path) {
                return Err(GustError::User(format!("Path {:?} is not a root path", absolute_path)));
            }
            if absolute_path.is_dir() {
                for file in self.process_folder(&absolute_path)? {
                    self.add_file(&file, &changed_files)?;
                }
            } else {
                self.add_file(&absolute_path, &changed_files)?;
            }
        }
        Ok(())
    }

    fn add_file(&mut self, file: &AbsolutePath, changed_files: &HashSet<RootRelativePath>) -> Result<()> {
        let relative_path = RootRelativePath::new(&file, &self.path);
        if changed_files.contains(&relative_path) {
            self.staging_area.insert(relative_path)?;
        }
        Ok(())
    }

    pub fn status(&self) -> Result<()> {
        let changed_files = self.get_changed_files()?;
        println!("Changes to be committed:");
        if self.staging_area.is_empty() {
            println!("  No changes");
        } else {
            for file in self.staging_area.into_iter() {
                println!("  {}", file.display())
            }
        }
        println!("\nUnstaged changes:");
        let mut unstaged_file_exists = false;
        for file in changed_files {
            if !self.staging_area.contains(&file) {
                println!("  {}", file.display());
                unstaged_file_exists = true;
            }
        }
        if !unstaged_file_exists {
            println!("  No changes");
        }
        Ok(())
    }

    fn get_changed_files(&self) -> Result<HashSet<RootRelativePath>> {
        // TODO: Check for deleted files
        let files = self.process_folder(&AbsolutePath::from_absolute_path(self.path.as_path()))?;
        let commit = Commit::from_commit_ref_option(self.branch.get_last_commit_ref(), &self.path)?;
        let mut changed_files: HashSet<RootRelativePath> = HashSet::new();

        for file in files {
            let relative_file_path = RootRelativePath::new(&file, &self.path);
            if let Some(c) = commit.as_ref() {
                if c.has_file_changed(&relative_file_path, &file)? {
                    changed_files.insert(relative_file_path);
                }
            } else {
                changed_files.insert(relative_file_path);
            }
        }
        Ok(changed_files)
    }

    fn process_folder(&self, path: &AbsolutePath) -> Result<Vec<AbsolutePath>> {
        if path.as_path() == &self.path.as_path().join(".gust") {
            return Ok(Vec::new()); // Dont process the root .gust folder
        }

        let entries = fs::read_dir(path.as_path()).unwrap(); // I know the path exists and is a dir
        let mut files = Vec::new();

        for entry in entries {
            let entry_path = AbsolutePath::from_absolute_path(&entry.unwrap().path());
            if entry_path.is_dir() {
                let entry_result = self.process_folder(&entry_path)?;
                files.extend(entry_result);
            } else {
                files.push(entry_path);
            }
        }
        Ok(files)
    }
}