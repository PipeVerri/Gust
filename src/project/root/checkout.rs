use std::collections::{HashMap};
use clap::ValueEnum;
use crate::project::branch::{Branch, BranchTrait, DetachedBranch};
use crate::project::commit::{Commit, CommitRef};
use super::Root;
use crate::project::error::{GustError, Result};
use crate::project::head::Head;
use crate::project::paths::RootRelativePath;
use crate::project::storable::{ContainsStorePath, ProjectStorable};
use std::fs;
use crate::project::tracked_file::TrackedFile;

#[derive(ValueEnum, Clone)]
pub(crate) enum CheckoutMode {
    Branch,
    Commit,
}

impl Root {
    pub fn checkout(&mut self, checkout_mode: &Option<CheckoutMode>, name: &str) -> Result<()> {
        if !self.get_changed_files()?.is_empty() {
            return Err(GustError::User(
                "There are uncommitted changes in the project. Commit or stash them before checking out a branch".into()
            ));
        }

        // TODO: Map the "file not found error" to "branch doesnt exist"
        if let Some(mode) = checkout_mode {
            match mode {
                CheckoutMode::Branch => self.checkout_branch(name)?,
                CheckoutMode::Commit => self.checkout_commit(name)?,
            };
        } else {
            // Automatically check the kind of checkout asked for
            unimplemented!("Auto checkout mode detector not implemented yet");
        }
        Ok(())
    }

    fn checkout_commit(&mut self, partial_hash: &str) -> Result<()> {
        let full_hash = self.get_full_commit_hash(partial_hash)?;
        let commit = Commit::load((self.path.clone(), full_hash.clone()))?;
        let commit_ref = CommitRef::new_from_existing(&commit, full_hash);

        // TODO: No hacer operaciones destructivas como handle_checkout en puntos donde puedan surgir errores
        // TODO: que hacer .save tambien se corra en los hijos
        self.apply_changes_to_working_tree(commit.copy_tree())?;
        self.head.handle_checkout()?;
        let detached_branch = DetachedBranch::new_from_commit_ref(commit_ref, &self.path, partial_hash)?;
        detached_branch.save()?;
        let new_head = Head::Detached(detached_branch);
        new_head.save_to_path(&Head::build_absolute_path(&self.path))?;
        self.head = new_head;
        Ok(())
    }

    fn get_full_commit_hash(&self, partial_hash: &str) -> Result<String> {
        let commits = fs::read_dir(&self.path.join(".gust/commits/").as_path())?;
        let mut found_commit_hashes= Vec::new();

        for commit in commits {
            let commit_name = commit?.path().file_stem().unwrap().to_str().unwrap().to_string();
            if commit_name.starts_with(partial_hash) {
                found_commit_hashes.push(commit_name);
            }
        }

        if found_commit_hashes.len() == 0 {
            Err(GustError::User("Commit not found".into()))
        } else if found_commit_hashes.len() == 1 {
            Ok(found_commit_hashes[0].clone())
        } else {
            Err(GustError::User(format!("Multiple commits found:\n{:?}", found_commit_hashes).into()))
        }
    }

    fn checkout_branch(&mut self, name: &str) -> Result<()> {
        let dest_branch = Branch::load((self.path.clone(), name.into()))?;
        let dest_branch_latest = Commit::from_commit_ref_option(dest_branch.get_last_commit_ref(), &self.path)?;
        let tree = if let Some(commit) = dest_branch_latest {
            commit.copy_tree()
        } else {
            HashMap::new()
        };

        self.apply_changes_to_working_tree(tree)?;

        let new_head = Head::Attached(dest_branch);
        new_head.save_to_path(&Head::build_absolute_path(&self.path))?;
        self.head.handle_checkout()?;
        self.head = new_head;
        Ok(())
    }

    fn apply_changes_to_working_tree(&self, target_tree: HashMap<RootRelativePath, TrackedFile>) -> Result<()> {
        // Delete files not present in the tree
        for absolute_path in self.scan_folder(&self.path.join("."))? {
            let relative_path = RootRelativePath::new(&absolute_path, &self.path)?;
            if !target_tree.contains_key(&relative_path) { // TODO: Check that the file is also not being ignored
                fs::remove_file(absolute_path.as_path())?;
            }
        }

        // Set the files with the tree's version of them
        for (path, content) in target_tree {
            let destination_path = &self.path.join(path.as_path());
            let blob_path = self.path.join(&format!(".gust/blobs/{}", content.get_blob_id()));
            fs::copy(blob_path.as_path(), destination_path.as_path())?;
        }

        Ok(())
    }
}