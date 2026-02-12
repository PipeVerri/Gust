use std::borrow::Cow;
use std::fs;
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
    pub name: String,
}

pub(super) struct DetachedBranch {
    commits: Vec<CommitRef>,
    store_path: AbsolutePath,
    pub passed_hash: String,
}

pub trait BranchTrait: ContainsStorePath {
    fn commits(&self) -> &Vec<CommitRef>;
    fn commits_mut(&mut self) -> &mut Vec<CommitRef>;
    fn store_path(&self) -> &AbsolutePath;
    fn get_last_commit_ref(&self) -> Option<&CommitRef> {
        if self.commits().is_empty() {
            None
        } else {
            Some(&self.commits()[self.commits().len() - 1])
        }
    }
    fn insert(&mut self, commit_ref: CommitRef) -> Result<()> {
        self.commits_mut().push(commit_ref);
        self.save()?;
        Ok(())
    }
    fn display(&self) -> String {
        let mut result = String::new();
        for commit in self.commits().iter().rev() {
            result.push_str(&commit.display());
            result.push('\n');
        }
        result
    }
    fn handle_checkout(&self) -> Result<()>;
    fn new_from_commit_ref(commit_ref: CommitRef, root_path: &RootPath, id: &str) -> Result<Self>;
}

// General implementation
impl<T: BranchTrait + ProjectStorable> ContainsStorePath for T {
    fn get_absolute_path(&self) -> &AbsolutePath { &self.store_path() }
}

// Branch implementation
impl BranchTrait for Branch {
    fn commits(&self) -> &Vec<CommitRef> { &self.commits }
    fn commits_mut(&mut self) -> &mut Vec<CommitRef> { &mut self.commits }
    fn store_path(&self) -> &AbsolutePath { &self.store_path }
    fn handle_checkout(&self) -> Result<()> { Ok(()) }
    fn new_from_commit_ref(commit_ref: CommitRef, root_path: &RootPath, id: &str) -> Result<Self> {
        Ok(Self {
            commits: vec![commit_ref],
            store_path: Self::build_absolute_path(&(root_path.clone(), id.to_string())),
            name: id.to_string()
        })
    }
}

impl ProjectStorable for Branch {
    type Stored = Vec<CommitRef>;
    type CreationArgs = (RootPath, String);
    fn build_absolute_path(creation_args: &Self::CreationArgs) -> AbsolutePath {
        creation_args.0.join(&format!(".gust/branches/{}.json", creation_args.1))
    }
    fn from_stored(stored: Self::Stored, creation_args: Self::CreationArgs) -> Result<Self> {
        Ok(Self { commits: stored, store_path: Self::build_absolute_path(&creation_args), name: creation_args.1 })
    }
    fn into_stored(&self) -> Cow<'_, Self::Stored> {
        Cow::Borrowed(&self.commits)
    }
}

impl Branch {
    pub fn new_from_tree(tree: Vec<CommitRef>, root_path: &RootPath, id: &str) -> Self {
        Self {
            commits: tree,
            store_path: Self::build_absolute_path(&(root_path.clone(), id.to_string())),
            name: id.to_string()
        }
    }
}

// DetachedBranch implementation

impl BranchTrait for DetachedBranch {
    fn commits(&self) -> &Vec<CommitRef> { &self.commits }
    fn commits_mut(&mut self) -> &mut Vec<CommitRef> { &mut self.commits }
    fn store_path(&self) -> &AbsolutePath { &self.store_path }
    fn handle_checkout(&self) -> Result<()> {
        fs::remove_file(self.store_path.as_path())?;
        Ok(())
    }
    fn new_from_commit_ref(commit_ref: CommitRef, root_path: &RootPath, id: &str) -> Result<Self> {
        Ok(Self {
            commits: vec![commit_ref],
            store_path: Self::build_absolute_path(root_path),
            passed_hash: id.to_string()
        })
    }
}

impl ProjectStorable for DetachedBranch {
    type Stored = (Vec<CommitRef>, String);
    type CreationArgs = RootPath;

    // ACA ESTA EL BUG, BUSCA EL BRANCH EN DETACHED_HEAD.json PERO EN REALIDAD ESTA EN branches/<passed_hash>.json
    fn build_absolute_path(creation_args: &Self::CreationArgs) -> AbsolutePath {
        creation_args.join(".gust/branches/DETACHED_HEAD.json")
    }
    fn from_stored(stored: Self::Stored, creation_args: Self::CreationArgs) -> Result<Self> {
        Ok(Self { commits: stored.0, store_path: Self::build_absolute_path(&creation_args), passed_hash: stored.1 })
    }
    fn into_stored(&self) -> Cow<'_, Self::Stored> {
        Cow::Owned((self.commits.clone(), self.passed_hash.clone()))
    }
}