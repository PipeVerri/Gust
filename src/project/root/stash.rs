use std::borrow::Cow;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use super::{Root, RootPath};
use crate::project::error::Result;
use crate::project::paths::{AbsolutePath, RootRelativePath};
use crate::project::storable::ProjectStorable;
use crate::project::tracked_file::TrackedFile;

#[derive(Serialize, Deserialize, Clone)]
pub(super) struct StashReference {
    id: u32,
    message: Option<String>,
}

pub(super) struct Stash {
    tree: HashMap<RootRelativePath, TrackedFile>,
}

pub(super) struct Stashes(Vec<StashReference>);

impl Root {
    pub fn save_stash(&mut self, message: Option<String>) -> Result<()> {
        let changed_files = self.get_changed_files()?;
        let stash_reference = Stashes::create(self.path.clone());
        Ok(())
    }

    pub fn apply_stash(&mut self, stash_id: u32) -> Result<()> {
        unimplemented!()
    }

    pub fn pop_stash(&mut self, stash_id: u32) -> Result<()> {
        unimplemented!()
    }

    pub fn drop_stash(&mut self, stash_id: u32) -> Result<()> {
        unimplemented!()
    }

    pub (super) fn get_stashes(&self) -> Result<Vec<u32>> {
        unimplemented!()
    }
}

impl ProjectStorable for Stashes {
    type Stored = Vec<StashReference>;
    type CreationArgs = RootPath;
    fn build_absolute_path(creation_args: &Self::CreationArgs) -> AbsolutePath {
        creation_args.join(".gust/stashes.json")
    }
    fn from_stored(stored: Self::Stored, _: Self::CreationArgs) -> Result<Self> {
        Ok(Self(stored))
    }
    fn into_stored(&self) -> Cow<'_, Self::Stored> {
        Cow::Borrowed(&self.0)
    }
}

impl ProjectStorable for Stash {
    type Stored = HashMap<RootRelativePath, TrackedFile>;
    type CreationArgs = (RootPath, StashReference);
    fn build_absolute_path(creation_args: &Self::CreationArgs) -> AbsolutePath {
        creation_args.0.join(format!(".gust/stashes/{}.json", creation_args.1.id))
    }
    fn from_stored(stored: Self::Stored, _: Self::CreationArgs) -> Result<Self> {
        Ok(Self { tree: stored })
    }
    fn into_stored(&self) -> Cow<'_, Self::Stored> {
        Cow::Borrowed(&self.tree)
    }
}