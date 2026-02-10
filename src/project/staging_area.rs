use super::paths::{AbsolutePath, RootRelativePath};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::root::RootPath;
use super::storable::{HasAbsolutePath, ProjectStorable, FixedStorable};
use super::error::{GustError, Result};

#[derive(Debug)]
pub(super) struct StagingArea {
    files: HashMap<RootRelativePath, ChangeType>,
    store_path: AbsolutePath,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Added,
    Modified,
    Removed
}

impl StagingArea {
    pub fn insert(&mut self, path: RootRelativePath, change: ChangeType) -> Result<()> {
        self.files.insert(path, change);
        self.save()
    }
    pub fn remove(&mut self, path: RootRelativePath) -> Result<()> {
        if self.files.contains_key(&path) {
            self.files.remove(&path);
            self.save()?;
            Ok(())
        } else {
            Err(GustError::User(format!("{} isn't in the staging area", path.display())))
        }
    }
    pub fn is_empty(&self) -> bool { self.files.is_empty() }
    pub fn contains(&self, path: &RootRelativePath) -> bool {
        self.files.contains_key(path)
    }
    pub fn get_files(&self) -> HashMap<RootRelativePath, ChangeType> {
        self.files.clone()
    }
    pub fn clear(&mut self) -> Result<()> {
        self.files.clear();
        self.save()?;
        Ok(())
    }
}

impl HasAbsolutePath for StagingArea {
    fn get_absolute_path(&self) -> &AbsolutePath {
        &self.store_path
    }
}

impl ProjectStorable for StagingArea {
    type Stored = HashMap<RootRelativePath, ChangeType>;
    fn from_stored(stored: Self::Stored, store_path: AbsolutePath) -> Self {
        Self { files: stored, store_path }
    }
    fn into_stored(&self) -> &Self::Stored {
        &self.files
    }
}

impl FixedStorable for StagingArea {
    fn create_absolute_path(path: &RootPath) -> AbsolutePath {
        path.join(".gust/staging_area.json")
    }
}

impl<'a> IntoIterator for &'a StagingArea {
    type Item = (&'a RootRelativePath, &'a ChangeType);
    type IntoIter = std::collections::hash_map::Iter<'a, RootRelativePath, ChangeType>;
    fn into_iter(self) -> Self::IntoIter { self.files.iter() }
}

impl ChangeType {
    pub fn display(&self) -> &'static str {
        match self {
            ChangeType::Added => "+",
            ChangeType::Modified => "~",
            ChangeType::Removed => "-"
        }
    }
}