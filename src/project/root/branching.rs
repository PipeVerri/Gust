use super::Root;
use super::Head;
use crate::project::error::Result;
use crate::project::branch::Branch;
use crate::project::storable::{ContainsStorePath, ProjectStorable};
use std::fs;

impl Root {
    pub fn branch(&mut self, branch_name: &Option<String>) -> Result<()> {
        if let Some(name) = branch_name {
            self.create_branch(name)
        } else {
            self.display_branches()
        }
    }

    fn display_branches(&self) -> Result<()> {
        let branches_path = self.path.join(".gust/branches");
        // TODO: Hacer un AsRef asi no tengo que poner .as_path todo el tiempo
        match &self.head {
            Head::Attached(branch) => {
                for branch_file in fs::read_dir(branches_path.as_path())? {
                    let branch_path = branch_file?.path();
                    match &self.head {
                        Head::Attached(branch) => {
                            if branch.get_absolute_path().as_path() != branch_path.as_path() {
                                println!("  {}", branch_path.file_stem().unwrap().to_str().unwrap());
                            }
                        },
                        Head::Detached => unimplemented!()
                    }
                }
                println!("* {}", branch.name);
            }
            Head::Detached => unimplemented!()
        }
        Ok(())
    }

    fn create_branch(&mut self, branch_name: &str) -> Result<()> {
        let current_head_commit = self.head.get_tree()?;
        let new_branch = if let Some(c) = current_head_commit {
            Branch::new_from_commit_ref(c.clone(), &self.path, branch_name)?
        } else {
            Branch::create((self.path.clone(), branch_name.to_string()))?
        };
        new_branch.save()?;
        Ok(())
    }
}