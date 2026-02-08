use std::fs;
use crate::project::paths::{AbsolutePath, CliPath, RootRelativePath};
use super::{Root, Result, GustError};

impl Root {
    pub fn add(&mut self, paths: Vec<CliPath>) -> Result<()> {
        for path in paths {
            let absolute_path = AbsolutePath::try_from(path)?; // Checks if it exists
            if !self.path.is_inside_root(&absolute_path) {
                return Err(GustError::User(format!("Path {:?} is not a root path", absolute_path)));
            }
            if absolute_path.is_dir() {
                for file in self.process_folder(&absolute_path)? {
                    self.staging_area.insert(RootRelativePath::new(&file, &self.path))?;
                }
            } else {
                self.staging_area.insert(RootRelativePath::new(&absolute_path, &self.path))?;
            }
        }
        Ok(())
    }

    pub fn status(&self) -> Result<()> {
        if self.staging_area.is_empty() {
            println!("Staging area: No changes.")
        } else {
            println!("Staging area:");
            for file in &self.staging_area {
                println!("  {}", file.display());
            }
        }
        println!("\nUnstaged changes:");
        Ok(())
        // TODO: show unstaged changes
    }

    fn process_folder(&mut self, path: &AbsolutePath) -> Result<Vec<AbsolutePath>> {
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