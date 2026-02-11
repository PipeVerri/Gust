use std::collections::HashMap;
use clap::ValueEnum;
use crate::project::branch::Branch;
use crate::project::commit::Commit;
use super::Root;
use crate::project::error::{GustError, Result};
use crate::project::head::Head;
use crate::project::paths::RootRelativePath;
use crate::project::storable::ProjectStorable;
use std::fs;

#[derive(ValueEnum, Clone)]
pub(crate) enum CheckoutMode {
    Branch,
    Commit,
}

impl Root {
    pub fn checkout(&mut self, checkout_mode: &Option<CheckoutMode>, name: &str) -> Result<()> {
        if let Some(mode) = checkout_mode {
            match mode {
                CheckoutMode::Branch => self.checkout_branch(name)?,
                CheckoutMode::Commit => unimplemented!()
            };
        } else {
            // Automatically check the kind of checkout asked for
            unimplemented!("Auto checkout mode detector not implemented yet");
        }
        Ok(())
    }

    fn checkout_branch(&mut self, name: &str) -> Result<()> {
        if !self.get_changed_files()?.is_empty() {
            return Err(GustError::User(
                "There are uncommitted changes in the project. Commit or stash them before checking out a branch".into()
            ));
        }
        let dest_branch = Branch::load((self.path.clone(), name.into()))?;
        let dest_branch_latest = Commit::from_commit_ref_option(dest_branch.get_last_commit_ref(), &self.path)?;
        let tree = if let Some(commit) = dest_branch_latest {
            commit.copy_tree()
        } else {
            HashMap::new()
        };

        // Delete files not present in the tree
        for absolute_path in self.scan_folder(&self.path.join("."))? {
            let relative_path = RootRelativePath::new(&absolute_path, &self.path)?;
            if !tree.contains_key(&relative_path) { // TODO: Check that the file is also not being ignored
                fs::remove_file(absolute_path.as_path())?;
            }
        }

        // Set the files with the tree's version of them
        for (path, content) in tree {
            let destination_path = &self.path.join_path(path.as_path());
            let blob_path = self.path.join(&format!(".gust/blobs/{}", content.get_blob_id()));
            fs::copy(blob_path.as_path(), destination_path.as_path())?;
        }

        let new_head = Head::Attached(dest_branch);
        new_head.save_to_path(&Head::build_absolute_path(&self.path))?;
        self.head = new_head;
        Ok(())
    }
}