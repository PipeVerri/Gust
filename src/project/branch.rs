use serde::{Serialize, Deserialize};
use crate::project::root::RootPath;
use super::commit::CommitRef;
use super::paths::AbsolutePath;
use super::storable::{ContainsStorePath, ProjectStorable};
use super::error::Result;

#[derive(Serialize, Deserialize)]
pub(super) struct Branch {
    commits: Vec<CommitRef>,
    store_path: AbsolutePath,
}

impl Branch {
    pub fn get_last_commit_ref(&self) -> Option<&CommitRef> {
        if self.commits.is_empty() {
            None
        } else {
            Some(&self.commits[self.commits.len() - 1])
        }
    }
    pub fn insert(&mut self, commit_ref: CommitRef) -> Result<()> {
        self.commits.push(commit_ref);
        self.save()?;
        Ok(())
    }
    pub fn display(&self) -> String {
        let mut result = String::new();
        for commit in self.commits.iter().rev() {
            result.push_str(&commit.display());
            result.push('\n');
        }
        result
    }
    pub fn new_from_commit_ref(commit_ref: CommitRef, root_path: &RootPath, id: &str) -> Result<Self> {
        let absolute_path = Branch::build_absolute_path(&(root_path.clone(), id.to_string()));
        let new_branch = Self {
            commits: vec![commit_ref],
            store_path: absolute_path,
        };
        Ok(new_branch)
    }
}

impl ProjectStorable for Branch {
    type Stored = Vec<CommitRef>;
    type CreationArgs = (RootPath, String);
    fn build_absolute_path(creation_args: &Self::CreationArgs) -> AbsolutePath {
        creation_args.0.join(&format!(".gust/branches/{}.json", creation_args.1))
    }
    fn from_stored(stored: Self::Stored, creation_args: Self::CreationArgs) -> Self {
        Self { commits: stored, store_path: Self::build_absolute_path(&creation_args) }
    }
    fn into_stored(&self) -> &Self::Stored {
        &self.commits
    }
}

impl ContainsStorePath for Branch {
    fn get_absolute_path(&self) -> &AbsolutePath {
        &self.store_path
    }
}