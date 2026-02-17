use std::collections::{HashMap};
use clap::ValueEnum;
use crate::project::branch::{Branch, BranchTrait, DetachedBranch};
use crate::project::commit::{Commit, CommitRef};
use super::Root;
use crate::project::error::{GustError, Result as GustResult};
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

enum CommitCheckoutError {
    CommitNotFound,
    MultipleCommitsFound(Vec<String>),
    NormalError(GustError)
}

impl From<std::io::Error> for CommitCheckoutError {
    fn from(value: std::io::Error) -> Self {
        CommitCheckoutError::NormalError(GustError::Io(value))
    }
}
impl From<CommitCheckoutError> for GustError {
    fn from(value: CommitCheckoutError) -> Self {
        match value {
            CommitCheckoutError::CommitNotFound => GustError::User("Commit not found".into()),
            CommitCheckoutError::MultipleCommitsFound(commits) => GustError::User(format!("Multiple commits found\n{:?}", commits)),
            CommitCheckoutError::NormalError(error) => error
        }
    }
}

impl Root {
    pub fn checkout(&mut self, checkout_mode: &Option<CheckoutMode>, name: &str) -> GustResult<()> {
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
            let branch_path = Branch::build_absolute_path(&(self.path.clone(), name.to_string()));
            let commit_hash = self.get_full_commit_hash(name);
            
            return if branch_path.as_path().exists() {
                match commit_hash {
                    Ok(_) => Err(GustError::User("Branch has the same name as commit. Specify if you want to checkout a branch or a commit using --mode".into())),
                    Err(e) => match e {
                        CommitCheckoutError::CommitNotFound => self.checkout_branch(name),
                        CommitCheckoutError::MultipleCommitsFound(found) => Err(GustError::User(format!("Branch name matches with multiple commits:\n{:?}\nSpecify if you want to checkout a branch or a commit using --mode", found))),
                        CommitCheckoutError::NormalError(error) => Err(error)
                    }
                }
            } else {
                match commit_hash {
                    Ok(_) => self.checkout_commit(name),
                    Err(e) => Err(e.into())
                }
            }
        }
        Ok(())
    }

    fn checkout_commit(&mut self, partial_hash: &str) -> GustResult<()> {
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

    fn get_full_commit_hash(&self, partial_hash: &str) -> Result<String, CommitCheckoutError> {
        let commits = fs::read_dir(&self.path.join(".gust/commits/").as_path())?;
        let mut found_commit_hashes= Vec::new();

        for commit in commits {
            let commit_name = commit?.path().file_stem().unwrap().to_str().unwrap().to_string();
            if commit_name.starts_with(partial_hash) {
                found_commit_hashes.push(commit_name);
            }
        }

        if found_commit_hashes.len() == 0 {
            Err(CommitCheckoutError::CommitNotFound)
        } else if found_commit_hashes.len() == 1 {
            Ok(found_commit_hashes[0].clone())
        } else {
            Err(CommitCheckoutError::MultipleCommitsFound(found_commit_hashes))
        }
    }

    fn checkout_branch(&mut self, name: &str) -> GustResult<()> {
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

    fn apply_changes_to_working_tree(&self, target_tree: HashMap<RootRelativePath, TrackedFile>) -> GustResult<()> {
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