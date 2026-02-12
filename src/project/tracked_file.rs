use std::time::SystemTime;
use serde::{Serialize, Deserialize};
use super::paths::AbsolutePath;
use super::error::{Result};
use std::fs;
use std::path::Path;
use crate::project::root::RootPath;

#[derive(Serialize, Deserialize, Clone)]
pub(super) struct TrackedFile {
    blob_id: String,
    pub metadata: Metadata
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
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
        let hash = hash_file(path.as_path())?;
        let blob_path = root_dir.join(&format!(".gust/blobs/{}", hash.to_string()));
        // Avoids re-copying the file's contents for duplicate files and doesn't raise an error for duplicate files
        if !blob_path.as_path().exists() {
            fs::copy(path.as_path(), blob_path.as_path())?;
        }

        // Create metadata
        let metadata = Metadata::new_from_file(path);
        if let Err(_) = metadata {
            fs::remove_file(blob_path.as_path())?;
        }

        Ok(Self {
            blob_id: hash,
            metadata: metadata?
        })
    }

    pub fn get_blob_id(&self) -> &str { &self.blob_id }
}

pub fn hash_file(path: &Path) -> Result<String> {
    let file = fs::read(path)?;
    Ok(sha256::digest(file))
}