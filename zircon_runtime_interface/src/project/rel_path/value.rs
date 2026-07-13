use std::fmt;
use std::path::{Path, PathBuf};

use serde::Serialize;

use super::RelPathError;

/// Portable normalized relative path that cannot escape its owning project root.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct RelPath(pub(super) String);

impl RelPath {
    pub fn project_assets() -> Self {
        Self("assets".to_string())
    }

    pub fn parse(value: impl AsRef<str>) -> Result<Self, RelPathError> {
        super::parse::parse(value.as_ref())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_path_buf(&self) -> PathBuf {
        self.0.split('/').collect()
    }

    pub fn join_to(&self, root: impl AsRef<Path>) -> PathBuf {
        root.as_ref().join(self.to_path_buf())
    }
}

impl fmt::Display for RelPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
