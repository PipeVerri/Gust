use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::project::root::RootPath;
use crate::project::staging_area::StagingArea;
use super::paths::{AbsolutePath, RootRelativePath};
use super::storable::{HasAbsolutePath, IdStorable, ProjectStorable};
use super::tracked_file::{Metadata, TrackedFile};
use super::error::{Result, GustError};

pub(super) struct Commit {
    store_path: AbsolutePath,
    data: StorableCommit
}

#[derive(Serialize, Deserialize)]
pub(super) struct CommitRef {
    commit_id: String
}

#[derive(Serialize, Deserialize, Default)]
pub(super) struct CommitMetadata {
    name: String
}

#[derive(Serialize, Deserialize, Default)]
pub(super) struct StorableCommit {
    tree: HashMap<RootRelativePath, TrackedFile>,
    metadata: CommitMetadata
}

impl HasAbsolutePath for Commit {
    fn get_absolute_path(&self) -> &AbsolutePath {
        &self.store_path
    }
}

impl ProjectStorable for Commit {
    type Stored = StorableCommit;
    fn from_stored(stored: Self::Stored, store_path: AbsolutePath) -> Self {
        Self {
            store_path,
            data: stored
        }
    }
    fn into_stored(&self) -> &Self::Stored {
        &self.data
    }

    fn handle_non_existence(path: &AbsolutePath) -> Result<Self::Stored> {
        Err(GustError::ProjectParsing(format!("Tried to load nonexistent commit at {:?}", path.as_path().display())))
    }
}

impl IdStorable for Commit {
    fn create_absolute_path(path: &RootPath, id: &str) -> AbsolutePath {
        path.join(&format!(".gust/commits/{}.json", id))
    }
}

impl Commit {
    pub fn from_commit_ref(reference: &CommitRef, root_path: &RootPath) -> Result<Commit> {
        let commit_path = Commit::create_absolute_path(root_path, &reference.commit_id);
        Commit::new_from_absolute(commit_path)
    }
    pub fn from_commit_ref_option(reference: Option<&CommitRef>, root_path: &RootPath) -> Result<Option<Commit>> {
        if let Some(commit_ref) = reference {
            Ok(Some(Self::from_commit_ref(commit_ref, root_path)?))
        } else {
            Ok(None)
        }
    }
    pub fn has_file_changed(&self, relative_path: &RootRelativePath, absolute_path: &AbsolutePath) -> Result<bool> {
        if let Some(tracked_file) = self.data.tree.get(relative_path) {
            return Ok(tracked_file.metadata != Metadata::new_from_file(absolute_path)?)
        }
        Ok(true) // If it wasn't present, it has been created, and it counts as a change
    }
}