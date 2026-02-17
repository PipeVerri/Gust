use std::borrow::Cow;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::project::root::{Root, RootPath};
use super::paths::{AbsolutePath, RootRelativePath};
use super::storable::{ContainsStorePath, ProjectStorable};
use super::tracked_file::{hash_file, Metadata, TrackedFile};
use super::error::{Result, GustError};
use super::staging_area::ChangeType;

pub(super) struct Commit {
    store_path: AbsolutePath,
    data: StorableCommit
}

#[derive(Serialize, Deserialize, Clone)]
pub(super) struct CommitRef {
    commit_id: String,
    metadata: CommitMetadata
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub(super) struct CommitMetadata {
    name: String
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub(super) struct StorableCommit {
    tree: HashMap<RootRelativePath, TrackedFile>,
    metadata: CommitMetadata
}

impl ProjectStorable for Commit {
    type Stored = StorableCommit;
    type CreationArgs = (RootPath, String);
    fn build_absolute_path(creation_args: &Self::CreationArgs) -> AbsolutePath {
        creation_args.0.join(&format!(".gust/commits/{}.json", creation_args.1))
    }
    fn from_stored(stored: Self::Stored, creation_args: Self::CreationArgs) -> Result<Self> {
        Ok(Self {
            store_path: Self::build_absolute_path(&creation_args),
            data: stored
        })
    }
    fn into_stored(&self) -> Cow<'_, Self::Stored> {
        Cow::Borrowed(&self.data)
    }

    fn handle_non_existence(path: &AbsolutePath) -> Result<Self::Stored> {
        Err(GustError::ProjectParsing(format!("Tried to load nonexistent commit at {:?}", path.as_path().display())))
    }
}

impl ContainsStorePath for Commit {
    fn get_absolute_path(&self) -> &AbsolutePath {
        &self.store_path
    }
}

pub enum FileStatus {
    Added,
    Modified,
    Unchanged
}

impl Commit {
    pub fn from_commit_ref(reference: &CommitRef, root_path: &RootPath) -> Result<Commit> {
        Commit::create((root_path.clone(), reference.commit_id.clone()))
    }
    pub fn from_commit_ref_option(reference: Option<&CommitRef>, root_path: &RootPath) -> Result<Option<Commit>> {
        if let Some(commit_ref) = reference {
            Ok(Some(Self::from_commit_ref(commit_ref, root_path)?))
        } else {
            Ok(None)
        }
    }
    pub fn has_file_changed(&self, relative_path: &RootRelativePath, absolute_path: &AbsolutePath) -> Result<FileStatus> {
        if let Some(tracked_file) = self.data.tree.get(relative_path) {
            return if tracked_file.metadata != Metadata::new_from_file(absolute_path)? { // Compare hashes if files aren't equal
                if hash_file(absolute_path.as_path())? != tracked_file.get_blob_id() {
                    Ok(FileStatus::Modified)
                } else {
                    Ok(FileStatus::Unchanged)
                }
            } else {
                Ok(FileStatus::Unchanged) // Same metadata = same file
            }
        }
        Ok(FileStatus::Added) // If it wasn't present, it has been created, and it counts as a change
    }

    pub fn tree_iterator(&self) -> std::collections::hash_map::Iter<'_, RootRelativePath, TrackedFile> {
        self.data.tree.iter()
    }

    pub fn copy_tree(&self) -> HashMap<RootRelativePath, TrackedFile> {
        self.data.tree.clone()
    }
}

impl CommitRef {
    pub fn new_commit(root: &Root, metadata: CommitMetadata) -> Result<CommitRef> {
        let mut tree = if let Some(c) = root.get_last_commit()? {
            c.copy_tree()
        } else {
            HashMap::new()
        };
        for (file, change_type) in root.get_staging_area().get_files() {
            match change_type {
                ChangeType::Removed => { tree.remove(&file); },
                _ => {
                    let absolute_file_path = root.get_path().join(file.as_path());
                    let tracked_file = TrackedFile::new(&absolute_file_path, root.get_path())?;
                    tree.insert(file, tracked_file);
                }
            };
        }
        let storable = StorableCommit {
            tree,
            metadata: metadata.clone()
        };
        let id = sha256::digest(serde_json::to_string(&storable)?);
        let commit = Commit {
            store_path: Commit::build_absolute_path(&(root.get_path().clone(), id.to_string())),
            data: storable
        };
        commit.save()?;
        Ok(CommitRef{ commit_id: id.to_string(), metadata })
    }
    
    pub fn new_from_existing(commit: &Commit, hash: String) -> Self {
        Self {
            commit_id: hash,
            metadata: commit.data.metadata.clone()
        }
    }
    
    pub fn display(&self) -> String { format!("{}: {}", self.metadata.name, self.commit_id) }
}

impl CommitMetadata {
    pub fn new(name: String) -> Self { Self { name } }
}