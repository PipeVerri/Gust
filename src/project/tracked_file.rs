use std::time::SystemTime;
use serde::{Serialize, Deserialize};
use super::paths::AbsolutePath;
use super::error::Result;
use std::fs;


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