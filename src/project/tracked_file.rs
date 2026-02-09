use std::time::SystemTime;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub(super) struct TrackedFile {
    blob_id: String,
    metadata: Metadata
}

#[derive(Serialize, Deserialize)]
struct Metadata {
    len: u64,
    modify_time: SystemTime,
    access_time: SystemTime
}