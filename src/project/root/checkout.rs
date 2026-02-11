use clap::ValueEnum;
use crate::project::branch::Branch;
use super::Root;
use crate::project::error::Result;
use crate::project::head::Head;
use crate::project::storable::ProjectStorable;

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
            unimplemented!();
        }
        Ok(())
    }

    fn checkout_branch(&mut self, name: &str) -> Result<()> {
        let new_head = Head::Attached(Branch::load((self.path.clone(), name.into()))?);
        new_head.save_to_path(&Head::build_absolute_path(&self.path))?;
        self.checkout_head(new_head);
        Ok(())
    }
}