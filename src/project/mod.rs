use std::{fs, io};
use std::path::PathBuf;
use crate::error::Result;

mod filesystem;
/*
Declares the existence of the filesystem module.
Cargo searches by default in the current dir for the filesystem.rs file, and copies its contents.
 */

pub struct Project {
    path: PathBuf,
}

impl Project {
    pub fn new() -> Result<Project> {
        let path = filesystem::find_project_root()?;
        Ok(Project { path })
    }

    pub fn print_path(&self) -> Result<()> {
        println!("{}", self.path.display());
        Ok(())
    }

    pub fn setup_project() -> Result<()> {
        // Create the dir and return IoError if it gets raised
        fs::create_dir("./.gust")?;
        Ok(())
    }
}