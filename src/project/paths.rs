use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};
use crate::project::root::ProjectRootPath;
use super::error::GustError;

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct CliPath(PathBuf);
#[derive(Serialize, Deserialize, Debug)]
pub(super) struct AbsolutePath(PathBuf);
#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq)]
pub(super) struct RootRelativePath(PathBuf);

impl From<&Path> for CliPath {
    fn from(path: &Path) -> Self { Self(path.into()) }
}

// Check if the CLI path exists before returning a non-existent absolute path
impl TryFrom<CliPath> for AbsolutePath {
    type Error = GustError;
    fn try_from(value: CliPath) -> Result<Self, Self::Error> {
        let cwd = std::env::current_dir()?;
        let joined = cwd.join(&value.0);
        if !joined.exists() {
            Err(GustError::User(format!("Path is not inside root: {}", joined.display())))
        } else {
            Ok(Self(joined))
        }
    }
}
impl AbsolutePath {
    pub fn from_absolute_path(path: &Path) -> Self { Self(path.into()) }
    pub fn as_path(&self) -> &Path { self.0.as_path() }
    pub fn strip_prefix(&self, prefix: &Path) -> &Path { self.0.strip_prefix(prefix).unwrap() }
    pub fn is_dir(&self) -> bool { self.0.is_dir() }
}

impl RootRelativePath {
    pub fn new(path: &AbsolutePath, root_path: &ProjectRootPath) -> RootRelativePath {
        Self(path.strip_prefix(root_path.as_path()).into())
    }
}