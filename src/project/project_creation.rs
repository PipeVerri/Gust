use std::{env, fs};
use std::io::Write;
use std::path::PathBuf;
use crate::error::{GustError, Result};
use crate::project::storables::Storable;
use crate::project::{Project, TrackedFile, StagingArea, Branch};

impl Project {
    pub fn new() -> Result<Project> {
        let path = find_project_root()?;
        let head = parse_head(&path)?;
        let branch = read_branch(&path, &head)?;
        let staging_area = StagingArea::new(path.join(".gust/staging_area.json").as_path())?;

        Ok(Project {
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

fn parse_head(root_path: &PathBuf) -> Result<String> {
    let head_path = root_path.join(".gust/HEAD");
    Ok(fs::read_to_string(head_path)?.trim().to_string())
}

fn read_branch(root_path: &PathBuf, head: &str) -> Result<Branch> {
    let branch_path = root_path.join(".gust/branches/").join(format!("{}.json", head));
    Branch::new(branch_path.as_path())
}

fn find_project_root() ->  Result<PathBuf> { // Size for Path needs to be known at compile time
    let mut path = env::current_dir()?;
    loop {
        path.push(".gust"); // Check if path/.gust exists
        if path.exists() {
            path.pop(); // Remove .gust
            return Ok(path);
        } else {
            // Go to the parent, pop twice because the path is now "parent/folder/.gust"
            path.pop();
            if !path.pop() { // Returns false when there isn't any parent
                return Err(GustError::User("No project root found".into()));
            }
        }
    }
}