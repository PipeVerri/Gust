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
    head: String,
    commits: Vec<String>,
    head_tree: Vec<TrackedFile>,
    staging_area: StagingArea,
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