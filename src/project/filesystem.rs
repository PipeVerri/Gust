use std::{env};
use std::io;
use std::path::PathBuf;

pub fn find_project_root() -> io::Result<PathBuf> {
    let mut path = env::current_dir()?;
    loop {
        path.push(".gust");
        if path.exists() {
            return Ok(path);
        } else {
            path.pop();
            if !path.pop() { // Returns false when there isn't any parent
                return Err(io::Error::new(io::ErrorKind::NotFound, "Unable to find project root"));
            }
        }
    }
}