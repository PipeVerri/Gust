use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::{fs, io};
use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::error::Result;
use super::StagingArea;

// The compiler needs to know Self is sized to know how much space it occupies on the stack
pub trait Storable: Sized {
    type Stored: Serialize + DeserializeOwned + Default; // Default allows the creation of a new empty instance
    fn from_stored(stored: Self::Stored, store_path: &Path) -> Self;
    fn into_stored(&self) -> &Self::Stored;
    fn new(path: &Path) -> Result<Self> {
        // If there isn't a file present, create a new Stored-typed default
        let stored: Self::Stored = match fs::File::open(path) {
            Ok(file) => serde_json::from_reader(file)?,
            Err(e) if e.kind() == io::ErrorKind::NotFound => Self::Stored::default(),
            Err(e) => return Err(e.into())
        };
        Ok(Self::from_stored(stored, path))
    }
    fn save(&self, path: &Path) -> Result<()> {
        let file = fs::File::create(path)?;
        serde_json::to_writer(file, &self.into_stored())?;
        Ok(())
    }
}

impl Storable for StagingArea {
    type Stored = HashSet<PathBuf>;
    fn from_stored(stored: Self::Stored, store_path: &Path) -> Self {
        StagingArea {files: stored, store_path: store_path.into() }
    }
    fn into_stored(&self) -> &Self::Stored {
        &self.files
    }
}