use super::Root;
use crate::project::error::Result;
use crate::project::branch::Branch;
use crate::project::storable::{ContainsStorePath, ProjectStorable};

impl Root {
    pub fn branch(&mut self, branch_name: &str) -> Result<()> {
        let current_head_commit = self.branch.get_last_commit_ref();
        let new_branch = if let Some(c) = current_head_commit {
            Branch::new_from_commit_ref(c.clone(), &self.path, branch_name)?
        } else {
            Branch::new((self.path.clone(), branch_name.to_string()))?
        };
        new_branch.save()?;
        Ok(())
    }
}