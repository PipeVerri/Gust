use serde::{Serialize, Deserialize};
use crate::project::paths::RootRelativePath;
use super::tracked_file::TrackedFile;

#[derive(Serialize, Deserialize)]
pub(super) struct Commit {
    tree: Vec<TrackedFile>,
    store_path: RootRelativePath,
}