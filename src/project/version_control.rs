use std::collections::HashSet;
use std::path::PathBuf;
use std::fs;
use super::{Project, StagingArea};
use crate::error::{Result, GustError};
use std::env;

impl Project {
    pub fn add(&mut self, paths: &Vec<PathBuf>) -> Result<()> {
        for path in paths {
            let path = make_path_absolute(path);
            self.check_path(&path)?;
            if path.is_dir() {
                self.process_folder(&path);
            } else {
                self.process_file(&path);
            }
        }
        println!("{:?}", self.staging_area);
        Ok(())
    }

    fn process_file(&mut self, path: &PathBuf) {
        let root_relative_path = self.make_path_relative_to_root(&path);
        self.staging_area.insert(root_relative_path);
    }

    fn process_folder(&mut self, path: &PathBuf) {
        let entries = fs::read_dir(path).unwrap(); // I know the path exists and is a dir
        for entry in entries {
            let entry_path = entry.unwrap().path();
            if entry_path.is_dir() {
                self.process_folder(&entry_path);
            } else {
                self.process_file(&entry_path);
            }
        }
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
    pub fn new() -> Self { StagingArea { files: HashSet::new() } }
    pub fn insert(&mut self, path: PathBuf) {
        self.files.insert(path);
        // TODO: save the staging area to disk
    }
}

fn make_path_absolute(path: &PathBuf) -> PathBuf {
    let cwd = env::current_dir().unwrap();
    cwd.join(path)
}