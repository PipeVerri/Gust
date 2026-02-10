use std::path::PathBuf;
use crate::project::commit::{CommitMetadata, CommitRef};
use crate::project::paths::{AbsolutePath, CliPath, RootRelativePath};
use super::{Root, Result};

impl Root {
    // CLI commands
    pub fn add(&mut self, paths: &Vec<PathBuf>) -> Result<()> {
        // TODO: Use a trie for faster addition
        // Checks that the user added the change either by passing the direct file or a parent directory
        for (file, change) in self.get_changed_files()? {
            let absolute_file = self.path.join_path(file.as_path());
            for cli_path in paths {
                let absolute_cli = AbsolutePath::try_from(CliPath::from(cli_path.as_path()))?;
                if absolute_file == absolute_cli || absolute_file.as_path().starts_with(absolute_cli.as_path()) {
                    self.staging_area.insert(file.clone(), change.clone())?;
                }
            }
        }

        Ok(())
    }

    pub fn remove(&mut self, paths: &Vec<PathBuf>) -> Result<()> {
        self.process_path_list(paths, |root, path| {
            let relative_path = RootRelativePath::new(&path, &root.path)?;
            root.staging_area.remove(relative_path)?;
            Ok(())
        })
    }

    pub fn status(&self) -> Result<()> {
        println!("Changes to be committed:");
        if self.staging_area.is_empty() {
            println!("  No changes");
        } else {
            for (file, change_type) in self.staging_area.into_iter() {
                println!("  {} {}", change_type.display(), file.display());
            }
        }

        println!("\nUnstaged changes:");
        let changed_files = self.get_changed_files()?;
        let mut unstaged_file_exists = false;
        for (file, change_type) in changed_files {
            if !self.staging_area.contains(&file) {
                println!("  {} {}", change_type.display(), file.display());
                unstaged_file_exists = true;
            }
        }
        if !unstaged_file_exists {
            println!("  No changes");
        }
        Ok(())
    }

    pub fn commit(&mut self, message: String) -> Result<()> {
        let metadata = CommitMetadata::new(message);
        let commit = CommitRef::new(&self, metadata)?;
        self.branch.insert(commit)?;
        self.staging_area.clear()?;
        Ok(())
    }

    pub fn info(&self) -> Result<()> {
        println!("Commit history:\n{}", self.branch.display());
        Ok(())
    }
}