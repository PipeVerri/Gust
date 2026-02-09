use serde::{Serialize, Deserialize};
use crate::project::root::RootPath;
use super::commit::CommitRef;
use super::paths::AbsolutePath;
use super::storable::{HasAbsolutePath, IdStorable, ProjectStorable};

#[derive(Serialize, Deserialize)]
pub(super) struct Branch {
    commits: Vec<CommitRef>,
    store_path: AbsolutePath
}

impl Branch {
    pub fn get_last_commit_ref(&self) -> Option<&CommitRef> {
        if self.commits.is_empty() {
            None
        } else {
            Some(&self.commits[self.commits.len() - 1])
        }
    }
}

impl HasAbsolutePath for Branch {
    fn get_absolute_path(&self) -> &AbsolutePath {
        &self.store_path
    }
}

impl ProjectStorable for Branch {
    type Stored = Vec<CommitRef>;
    fn from_stored(stored: Self::Stored, store_path: AbsolutePath) -> Self {
        Self { commits: stored, store_path }
    }
    fn into_stored(&self) -> &Self::Stored {
        &self.commits
    }
}

impl IdStorable for Branch {
    fn create_absolute_path(path: &RootPath, id: &str) -> AbsolutePath {
        path.join(&format!(".gust/branches/{}.json", id))
    }
}