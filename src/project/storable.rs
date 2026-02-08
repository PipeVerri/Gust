use std::{fs, io};
use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::error::Result;
use crate::project::paths::AbsolutePath;
use crate::project::root::ProjectRootPath;

pub trait HasAbsolutePath {
    fn get_absolute_path(&self) -> &AbsolutePath;
}

// The compiler needs to know Self is sized to know how much space it occupies on the stack
pub trait ProjectStorable: Sized + HasAbsolutePath {
    type Stored: Serialize + DeserializeOwned + Default; // Default allows the creation of a new empty instance
    fn from_stored(stored: Self::Stored, store_path: AbsolutePath) -> Self;
    fn into_stored(&self) -> &Self::Stored;
    fn new_from_absolute(path: AbsolutePath) -> Result<Self> {
        // If there isn't a file present, create a new Stored-typed default
        let stored: Self::Stored = match fs::File::open(path.as_path()) {
            Ok(file) => serde_json::from_reader(file)?,
            Err(e) if e.kind() == io::ErrorKind::NotFound => Self::Stored::default(),
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
    fn create_absolute_path(path: &ProjectRootPath) -> AbsolutePath;
    fn new_from_root(path: &ProjectRootPath) -> Result<Self> {
        Self::new_from_absolute(Self::create_absolute_path(path))
    }
}

pub trait IdStorable: ProjectStorable {
    fn create_absolute_path(path: &ProjectRootPath, id: &str) -> AbsolutePath;
    fn new_from_root(path: &ProjectRootPath, id: &str) -> Result<Self> {
        Self::new_from_absolute(Self::create_absolute_path(path, id))
    }
}

/*impl Storable for StagingArea {
    type Stored = HashSet<PathBuf>;
    fn from_stored(stored: Self::Stored, store_path: &Path) -> Self {
        StagingArea {files: stored, store_path: store_path.into() }
    }
    fn into_stored(&self) -> &Self::Stored {
        &self.files
    }
}

impl Storable for Branch {
    type Stored = Vec<Commit>;
    fn from_stored(stored: Self::Stored, store_path: &Path) -> Self {
        Self {
            commits: stored,
            store_path: store_path.into()
        }
    }

    fn into_stored(&self) -> &Self::Stored {
        &self.commits
    }
}

impl Storable for Commit {
    type Stored = Vec<TrackedFile>;
    fn from_stored(stored: Self::Stored, store_path: &Path) -> Self {
        Self {
            tree: stored,
            store_path: store_path.into()
        }
    }
    fn into_stored(&self) -> &Self::Stored {
        &self.tree
    }
}*/