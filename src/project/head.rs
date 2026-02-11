use std::borrow::Cow;
use crate::project::branch::Branch;
use crate::project::root::RootPath;
use crate::project::error::Result;
use crate::project::storable::ProjectStorable;
use serde::{Serialize, Deserialize};
use crate::project::commit::{CommitRef};
use crate::project::paths::AbsolutePath;

pub enum Head {
    Attached(Branch),
    Detached,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum StoredHead {
    Attached(String),
    Detached,
}

impl Default for StoredHead {
    fn default() -> Self { StoredHead::Attached("main".into()) }
}

impl Head {
    pub fn get_tree(&self) -> Result<Option<&CommitRef>> {
        Ok(match self {
            Self::Attached(branch) => branch.get_last_commit_ref(),
            Self::Detached => unimplemented!()
        })
    }

    pub fn insert_commit(&mut self, commit_ref: CommitRef) -> Result<()> {
        match self {
            Self::Attached(branch) => branch.insert(commit_ref),
            Self::Detached => unimplemented!()
        }
    }

    pub fn display(&self) -> String {
        match self {
            Self::Attached(branch) => {
                format!("Commit history of {} branch:\n", branch.name).to_string() +  branch.display().as_str()
            },
            Self::Detached => "Detached".into()
        }
    }

    fn default(root_path: &RootPath) -> Result<Self> {
        Ok(Self::Attached(Branch::create((root_path.clone(), "main".into()))?))
    }
}

impl ProjectStorable for Head {
    type Stored = StoredHead;
    type CreationArgs = RootPath;
    fn build_absolute_path(creation_args: &Self::CreationArgs) -> AbsolutePath {
        creation_args.join(".gust/HEAD.json")
    }
    fn from_stored(stored: Self::Stored, creation_args: Self::CreationArgs) -> Result<Head> {
        Ok(match stored {
            StoredHead::Attached(name) => Head::Attached(Branch::create((creation_args, name))?),
            StoredHead::Detached => unimplemented!()
        })
    }
    fn into_stored(&self) -> Cow<'_, Self::Stored> {
        Cow::Owned(match self {
            Self::Attached(branch) => StoredHead::Attached(branch.name.clone()),
            Self::Detached => StoredHead::Detached
        })
    }
}