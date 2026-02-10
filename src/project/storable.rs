use std::{fs, io};
use serde::de::DeserializeOwned;
use serde::Serialize;
use super::error::Result;
use crate::project::paths::AbsolutePath;
use crate::project::root::RootPath;

pub trait HasAbsolutePath {
    fn get_absolute_path(&self) -> &AbsolutePath;
}

// The compiler needs to know Self is sized to know how much space it occupies on the stack
pub trait ProjectStorable: Sized + HasAbsolutePath {
    type Stored: Serialize + DeserializeOwned + Default; // Default allows the creation of a new empty instance
    fn from_stored(stored: Self::Stored, store_path: AbsolutePath) -> Self;
    fn into_stored(&self) -> &Self::Stored;
    fn handle_non_existence(_: &AbsolutePath) -> Result<Self::Stored> {
        Ok(Self::Stored::default())
    }
    fn new_from_absolute(path: AbsolutePath) -> Result<Self> {
        // If there isn't a file present, create a new Stored-typed default
        let stored: Self::Stored = match fs::File::open(path.as_path()) {
            Ok(file) => serde_json::from_reader(file)?,
            Err(e) if e.kind() == io::ErrorKind::NotFound => Self::handle_non_existence(&path)?,
            Err(e) => return Err(e.into())
        };
        Ok(Self::from_stored(stored, path))
    }
    fn save(&self) -> Result<()> {
        let file = fs::File::create(self.get_absolute_path().as_path())?;
        serde_json::to_writer(file, &self.into_stored())?;
        Ok(())
    }
}

pub trait FixedStorable: ProjectStorable {
    fn create_absolute_path(path: &RootPath) -> AbsolutePath;
    fn new_from_root(path: &RootPath) -> Result<Self> {
        Self::new_from_absolute(Self::create_absolute_path(path))
    }
}

pub trait IdStorable: ProjectStorable {
    fn construct_absolute_path(path: &RootPath, id: &str) -> AbsolutePath;
    fn new_from_root(path: &RootPath, id: &str) -> Result<Self> {
        Self::new_from_absolute(Self::construct_absolute_path(path, id))
    }
}