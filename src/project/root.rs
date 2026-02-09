mod version_control;

use std::path::{Path, PathBuf};
use std::fs;
use std::env;
use std::io::Write;
use crate::project::paths::AbsolutePath;
use super::branch::Branch;
use super::staging_area::StagingArea;
use super::error::{Result, GustError};
use super::storable::{IdStorable, FixedStorable};

pub struct Root {
    path: RootPath,
    branch: Branch,
    staging_area: StagingArea,
}

impl Root {
    pub fn new() -> Result<Root> {
        let path = find_project_root()?;
        let head = parse_head(&path)?;
        let branch = Branch::new_from_root(&path, &head)?;
        let staging_area = StagingArea::new_from_root(&path)?;

        Ok(Root {
            path,
            branch,
            staging_area
        })
    }

    pub fn create_project() -> Result<()> {
        // Create the dir and return IoError if it gets raised
        fs::create_dir("./.gust")?;
        fs::create_dir("./.gust/blobs")?;
        fs::create_dir("./.gust/commits")?;
        fs::create_dir("./.gust/branches")?;
        fs::File::create("./.gust/branches/main")?;
        let mut head = fs::File::create("./.gust/HEAD")?;
        head.write_all(b"main")?;
        Ok(())
    }
}

fn find_project_root() ->  Result<RootPath> { // Size for Path needs to be known at compile time
    let mut path = env::current_dir()?;
    loop {
        path.push(".gust"); // Check if path/.gust exists
        if path.exists() {
            path.pop(); // Remove .gust
            // Doesn't use any constructors because this function should be the only one able to create a ProjectRootPath
            return Ok(RootPath(path));
        } else {
            // Go to the parent, pop twice because the path is now "parent/folder/.gust"
            path.pop();
            if !path.pop() { // Returns false when there isn't any parent
                return Err(GustError::User("No project found".into()));
            }
        }
    }
}

fn parse_head(root_path: &RootPath) -> Result<String> {
    let head_path = root_path.join(".gust/HEAD");
    Ok(fs::read_to_string(head_path.as_path())?.trim().to_string())
}

pub struct RootPath(PathBuf);
impl RootPath {
    pub(super) fn join(&self, path: &str) -> AbsolutePath { AbsolutePath::from_absolute_path(&self.0.join(path)) }
    pub(super) fn as_path(&self) -> &Path { self.0.as_path() }
    pub fn is_inside_root(&self, path: &AbsolutePath) -> bool {
        path.as_path().starts_with(&self.0.as_path())
    }
}