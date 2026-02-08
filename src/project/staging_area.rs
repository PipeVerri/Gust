use super::paths::{AbsolutePath, RootRelativePath};
use std::collections::HashSet;
use super::root::ProjectRootPath;
use super::storable::{HasAbsolutePath, ProjectStorable, FixedStorable};
use super::error::Result;

#[derive(Debug)]
pub(super) struct StagingArea {
    files: HashSet<RootRelativePath>,
    store_path: AbsolutePath,
}

impl StagingArea {
    pub fn insert(&mut self, path: RootRelativePath) -> Result<()> {
        self.files.insert(path);
        self.save()
    }
}

impl HasAbsolutePath for StagingArea {
    fn get_absolute_path(&self) -> &AbsolutePath {
        &self.store_path
    }
}

impl ProjectStorable for StagingArea {
    type Stored = HashSet<RootRelativePath>;
    fn from_stored(stored: Self::Stored, store_path: AbsolutePath) -> Self {
        Self { files: stored, store_path }
    }
    fn into_stored(&self) -> &Self::Stored {
        &self.files
    }
}

impl FixedStorable for StagingArea {
    fn create_absolute_path(path: &ProjectRootPath) -> AbsolutePath {
        path.join(".gust/staging_area.json")
    }
}