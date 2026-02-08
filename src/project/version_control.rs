use std::path::PathBuf;
use std::fs;
use super::{Project, StagingArea};
use crate::error::{Result, GustError};
use std::env;
use crate::project::storables::Storable;

impl Project {
    pub fn add(&mut self, paths: &Vec<PathBuf>) -> Result<()> {
        for path in paths {
            let path = make_path_absolute(path);
            self.check_path(&path)?;
            if path.is_dir() {
                for file in self.process_folder(&path)? {
                    self.staging_area.insert(self.make_path_relative_to_root(&file))?;
                }
            } else {
                self.staging_area.insert(self.make_path_relative_to_root(&path))?;
            }
        }

        println!("{:?}", self.staging_area);
        Ok(())
    }

    pub fn status(&self) -> Result<()> {
        if self.staging_area.files.is_empty() {
            println!("Staging area: No changes.")
        } else {
            println!("Staging area:");
            for file in &self.staging_area.files {
                println!("  {}", file.display());
            }
        }
        println!("\nUnstaged changes:");
        // TODO: show unstaged changes
        Ok(())
    }

    fn process_folder(&mut self, path: &PathBuf) -> Result<Vec<PathBuf>> {
        if (path == &self.path.join(".gust")) {
            return Ok(Vec::new()); // Dont process the root .gust folder
        }

        let entries = fs::read_dir(path).unwrap(); // I know the path exists and is a dir
        let mut files = Vec::new();

        for entry in entries {
            let entry_path = entry.unwrap().path();
            if entry_path.is_dir() {
                let entry_result = self.process_folder(&entry_path)?;
                files.extend(entry_result);
            } else {
                files.push(entry_path);
            }
        }
        Ok(files)
    }

    fn check_path(&self, path: &PathBuf) -> Result<()> {
        if !path.starts_with(&self.path) {
            return Err(GustError::User(format!("{} is not inside the project", path.display())))
        } else if !path.exists() {
            return Err(GustError::User(format!("{} does not exist", path.display())))
        }
        Ok(())
    }

    fn make_path_relative_to_root(&self, path: &PathBuf) -> PathBuf {
        // I dont know the lifetime of the &PathBuf
        path.strip_prefix(&self.path).unwrap().into() // unwrap() is safe because the path is within the project
    }
}

impl StagingArea {
    pub fn insert(&mut self, path: PathBuf) -> Result<()> {
        self.files.insert(path);
        self.save(&self.store_path)
    }
}

fn make_path_absolute(path: &PathBuf) -> PathBuf {
    let cwd = env::current_dir().unwrap();
    cwd.join(path)
}