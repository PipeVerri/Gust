use std::path::Path;
use serde::{Serialize, Deserialize};
use crate::project::root::ProjectRootPath;
use super::commit::Commit;
use super::paths::AbsolutePath;
use super::storable::{HasAbsolutePath, IdStorable, ProjectStorable};

#[derive(Serialize, Deserialize)]
pub(super) struct Branch {
    commits: Vec<Commit>,
    store_path: AbsolutePath
}
/*fn read_branch(root_path: &PathBuf, head: &str) -> Result<Branch> {
    let branch_path = root_path.join(".gust/branches/").join(format!("{}.json", head));
    Branch::new(branch_path.as_path())
}*/

impl HasAbsolutePath for Branch {
    fn get_absolute_path(&self) -> &AbsolutePath {
        &self.store_path
    }
}

impl ProjectStorable for Branch {
    type Stored = Vec<Commit>;
    fn from_stored(stored: Self::Stored, store_path: AbsolutePath) -> Self {
        Self { commits: stored, store_path }
    }
    fn into_stored(&self) -> &Self::Stored {
        &self.commits
    }
}

impl IdStorable for Branch {
    fn create_absolute_path(path: &ProjectRootPath, id: &str) -> AbsolutePath {
        path.join(&format!(".gust/branches/{}.json", id))
    }
}