use serde::{Serialize, Deserialize};
use crate::project::paths::RootRelativePath;

#[derive(Serialize, Deserialize)]
pub(super) struct TrackedFile {
    path: RootRelativePath,
    blob_id: String,
}