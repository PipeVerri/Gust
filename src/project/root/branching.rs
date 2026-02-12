use super::Root;
use super::Head;
use crate::project::error::Result;
use crate::project::branch::{Branch, BranchTrait};
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
        let current_branch_name = match &self.head {
            Head::Attached(branch) => &branch.name,
            Head::Detached(branch) => {
                println!("{}", format!("* HEAD attached at {}", branch.passed_hash));
                ""
            }
        };

        for branch_file in fs::read_dir(branches_path.as_path())? {
            let branch_name = branch_file?.path().file_stem().unwrap().to_str().unwrap().to_string();
            if branch_name != "DETACHED_HEAD" {
                if branch_name != current_branch_name {
                    println!("{}", branch_name);
                } else {
                    println!("* {}", branch_name);
                }
            }
        }
        Ok(())
    }

    fn create_branch(&mut self, branch_name: &str) -> Result<()> {
        match &self.head {
            Head::Attached(branch) => {
                let current_head_commit = branch.get_last_commit_ref();
                let new_branch = if let Some(c) = current_head_commit {
                    Branch::new_from_commit_ref(c.clone(), &self.path, branch_name)?
                } else {
                    Branch::create((self.path.clone(), branch_name.to_string()))?
                };
                new_branch.save()?;
            }
            Head::Detached(branch) => {
                let tree = branch.commits();
                let new_branch = Branch::new_from_tree(tree.clone(), &self.path, branch_name);
                new_branch.save()?;
            }
        };
        Ok(())
    }
}