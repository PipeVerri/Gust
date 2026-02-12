use std::borrow::Cow;
use super::paths::{AbsolutePath, RootRelativePath};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::root::RootPath;
use super::storable::{ContainsStorePath, ProjectStorable};
use super::error::Result;

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
        // I don't care if the user is trying to remove a file that isn't added to the staging area
        self.files.remove(&path);
        self.save()?;
        Ok(())
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

impl ProjectStorable for StagingArea {
    type Stored = HashMap<RootRelativePath, ChangeType>;
    type CreationArgs = RootPath;
    fn build_absolute_path(creation_args: &Self::CreationArgs) -> AbsolutePath {
        creation_args.join(".gust/staging_area.json")
    }
    fn from_stored(mut stored: Self::Stored, creation_args: Self::CreationArgs) -> Result<Self> {
        // If a file was modified or added, and now it doesn't exist anymore, remove it from the staging area
        stored.retain(|path, change_type| {
            // If I'm staging a removal, then the file won't exist, but I still want the change in the staging area
            match change_type {
                ChangeType::Removed => true,
                _ => {
                    let absolute_path = creation_args.join(path.as_path());
                    absolute_path.as_path().exists()
                }
            }
        });
        Ok(Self { files: stored, store_path: Self::build_absolute_path(&creation_args) })
    }
    fn into_stored(&self) -> Cow<'_, Self::Stored> {
        Cow::Borrowed(&self.files)
    }
}

impl ContainsStorePath for StagingArea {
    fn get_absolute_path(&self) -> &AbsolutePath {
        &self.store_path
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
            ChangeType::Modified => "M",
            ChangeType::Removed => "-"
        }
    }
}