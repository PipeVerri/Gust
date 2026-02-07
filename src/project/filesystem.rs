use std::{env};
use std::path::PathBuf;
use crate::error::{GustError, Result};

pub fn find_project_root() ->  Result<PathBuf>{
    let mut path = env::current_dir()?;
    loop {
        path.push(".gust");
        if path.exists() {
            return Ok(path);
        } else {
            path.pop();
            if !path.pop() { // Returns false when there isn't any parent
                return Err(GustError::Project("No project root found".into()));
            }
        }
    }
}