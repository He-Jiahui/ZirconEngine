use std::cmp::Ordering;
use std::path::{Path, PathBuf};

/// Ordered unresolved identity for frozen source sets; construction never probes the filesystem.
#[derive(Clone, Debug)]
pub(crate) struct LexicalProjectPathIdentity {
    path: PathBuf,
}

impl LexicalProjectPathIdentity {
    pub(super) fn new(path: &Path) -> Result<Self, std::io::Error> {
        let path = super::absolute_project_path(path)?;
        #[cfg(windows)]
        let path = super::windows::normalize_windows_final_path(path);
        Ok(Self { path })
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl PartialEq for LexicalProjectPathIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for LexicalProjectPathIdentity {}

impl PartialOrd for LexicalProjectPathIdentity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LexicalProjectPathIdentity {
    fn cmp(&self, other: &Self) -> Ordering {
        #[cfg(windows)]
        {
            super::windows::compare_paths_ignore_case(&self.path, &other.path)
        }
        #[cfg(not(windows))]
        {
            self.path.cmp(&other.path)
        }
    }
}

#[cfg(test)]
#[path = "tests/lexical_identity.rs"]
mod tests;
