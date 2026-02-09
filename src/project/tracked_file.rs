use std::time::SystemTime;
use serde::{Serialize, Deserialize};
use super::paths::AbsolutePath;
use super::error::{GustError, Result};
use std::fs;
use crate::project::root::RootPath;

#[derive(Serialize, Deserialize)]
pub(super) struct TrackedFile {
    blob_id: String,
    pub metadata: Metadata
}

#[derive(Serialize, Deserialize, PartialEq)]
pub(super) struct Metadata {
    len: u64,
    modify_time: SystemTime,
    access_time: SystemTime
}

impl Metadata {
    pub fn new_from_file(path: &AbsolutePath) -> Result<Self> {
        let metadata = fs::metadata(path.as_path())?;
        Ok(Self { len: metadata.len(), modify_time: metadata.modified()?, access_time: metadata.accessed()? })
    }
}

impl TrackedFile {
    pub fn new(path: &AbsolutePath, root_dir: &RootPath) -> Result<Self> {
        // Create the blob
        let file = fs::read(path.as_path())?;
        let hash = sha256::digest(file);
        let blob_path = root_dir.join(&format!(".gust/blobs/{}", hash.to_string()));
        if blob_path.as_path().exists() {
            return Err(GustError::ProjectParsing(format!("Blob for {} already exists", path.as_path().display())));
        }
        fs::copy(path.as_path(), blob_path.as_path())?;

        // Create metadata
        let metadata = Metadata::new_from_file(path);
        if let Err(e) = &metadata {
            fs::remove_file(blob_path.as_path())?;
        }

        Ok(Self {
            blob_id: hash,
            metadata: metadata?
        })
    }
}