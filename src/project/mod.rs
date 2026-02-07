use std::{fs, io};
use std::path::PathBuf;

mod filesystem;
/*
Declares the existence of the filesystem module.
Cargo searches by default in the current dir for the filesystem.rs file, and copies its contents.
 */

pub struct Project {
    path: PathBuf,
}

impl Project {
    pub fn new() -> std::io::Result<Project> {
        let path = filesystem::find_project_root()?;
        Ok(Project { path })
    }
    
    pub fn setup_project() -> io::Result<()> {
        fs::create_dir("./.gust")
    }
}