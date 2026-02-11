use std::{fs, io};
use std::borrow::Cow;
use serde::de::DeserializeOwned;
use serde::Serialize;
use super::error::Result;
use crate::project::paths::AbsolutePath;
use crate::project::root::RootPath;

// The compiler needs to know Self is sized to know how much space it occupies on the stack
pub trait ProjectStorable: Sized {
    type Stored: Serialize + DeserializeOwned + Default + Clone; // Default allows the creation of a new empty instance, Clone so Cow can take ownership
    type CreationArgs;
    fn build_absolute_path(creation_args: &Self::CreationArgs) -> AbsolutePath;
    fn from_stored(stored: Self::Stored, creation_args: Self::CreationArgs) -> Self;
    fn into_stored(&self) -> Cow<'_, Self::Stored>;
    fn handle_non_existence(_: &AbsolutePath) -> Result<Self::Stored> {
        Ok(Self::Stored::default())
    }
    fn new(creation_args: Self::CreationArgs) -> Result<Self> {
        // If there isn't a file present, create a new Stored-typed default
        let stored: Self::Stored = match fs::File::open(Self::build_absolute_path(&creation_args).as_path()) {
            Ok(file) => serde_json::from_reader(file)?,
            Err(e) if e.kind() == io::ErrorKind::NotFound => Self::handle_non_existence(&Self::build_absolute_path(&creation_args))?,
            Err(e) => return Err(e.into())
        };
        Ok(Self::from_stored(stored, creation_args))
    }
    fn save_to_path(&self, path: &AbsolutePath) -> Result<()> {
        let file = fs::File::create(path.as_path())?;
        serde_json::to_writer(file, &self.into_stored())?;
        Ok(())
    }
}

pub trait ContainsStorePath: ProjectStorable {
    fn get_absolute_path(&self) -> &AbsolutePath;
    fn save(&self) -> Result<()> {
        let file = fs::File::create(self.get_absolute_path().as_path())?;
        serde_json::to_writer(file, &self.into_stored())?;
        Ok(())
    }
}

// The creation of AbsolutePaths from RootPath will be in the implementation of CreationArgs
// For both RootStorable and IdStorable