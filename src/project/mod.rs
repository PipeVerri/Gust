use std::collections::HashSet;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

mod project_creation;
mod version_control;
mod storables;
/*
Declares the existence of the filesystem module.
Cargo searches by default in the current dir for the project_creation file and copies its contents.
 */

pub struct Project {
    path: PathBuf,
    branch: Branch,
    staging_area: StagingArea,
}

#[derive(Serialize, Deserialize)]
struct Branch {
    commits: Vec<Commit>,
    store_path: PathBuf
}

#[derive(Serialize, Deserialize)]
struct Commit {
    tree: Vec<TrackedFile>,
    store_path: PathBuf
}

#[derive(Serialize, Deserialize)]
struct TrackedFile {
    path: PathBuf,
    blob_id: String,
}

#[derive(Debug)]
struct StagingArea {
    files: HashSet<PathBuf>,
    store_path: PathBuf
}

impl Branch {
    fn head(&self) -> Option<&Commit> {
        if self.commits.is_empty() {
            None
        } else {
            Some(&self.commits[self.commits.len() - 1])
        }
    }
}