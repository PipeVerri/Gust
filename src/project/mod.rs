use std::path::PathBuf;
use crate::error::Result;
use serde::{Deserialize, Serialize};

mod project_creation;
/*
Declares the existence of the filesystem module.
Cargo searches by default in the current dir for the project_creation file, and copies its contents.
 */

pub struct Project {
    path: PathBuf,
    head: String,
    commits: Vec<String>,
    head_tree: Vec<TrackedFile>,
}

#[derive(Serialize, Deserialize)]
struct TrackedFile {
    path: PathBuf,
    blob_id: String,
}

impl Project {
    pub fn print_path(&self) -> Result<()> {
        println!("{}", self.path.display());
        Ok(())
    }
}