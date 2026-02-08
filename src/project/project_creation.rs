use std::{env, fs};
use std::io::Write;
use std::path::PathBuf;
use crate::error::{GustError, Result};
use super::{Project, TrackedFile};

impl Project {
    pub fn new() -> Result<Project> {
        let path = find_project_root()?;
        let head = parse_head(&path)?;
        let commits = read_branch(&path, &head)?;
        let tree = if commits.is_empty() {
            Vec::new()
        } else {
            read_commit(&path, &commits.last().unwrap())? // Returns an option in the case the vector is empty
        };

        Ok(Project { 
            path, 
            head, 
            commits, 
            head_tree: tree 
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

fn read_commit(root_path: &PathBuf, commit: &str) -> Result<Vec<TrackedFile>> {
    // Open tree.json
    let tree_path = root_path.join(".gust/commits/").join(commit).join("tree.json");
    let tree_raw = fs::File::open(tree_path)?;

    // Deserialize it and check for errors
    let tree = serde_json::from_reader(tree_raw);
    if let Err(_) = tree {
        return Err(GustError::Project("Invalid commit tree".into()));
    }

    // Convert it to a vector of TrackedFile structs
    let tree_vec: Vec<TrackedFile> = tree.unwrap();
    Ok(tree_vec)
}

fn parse_head(root_path: &PathBuf) -> Result<String> {
    let head_path = root_path.join(".gust/HEAD");
    Ok(fs::read_to_string(head_path)?.trim().to_string())
}

fn read_branch(root_path: &PathBuf, head: &str) -> Result<Vec<String>> {
    let branch_path = root_path.join(".gust/branches/").join(head);
    let branch = fs::read_to_string(branch_path)?;
    Ok(branch.
        lines()// Create an iterator based on newlines
        .filter(|line| !line.is_empty()) // Filter empty lines
        .map(|line| line.to_string()) // Convert them to Strings
        .collect()) // Collect the iterator into a Vec
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
                return Err(GustError::Project("No project root found".into()));
            }
        }
    }
}