use std::cmp::Ordering;
use std::path::Path;

use super::ResolvedProjectPath;

/// Ordered identity for coordinating access to one resolved project path.
///
/// The retained path remains suitable for filesystem operations. Equality and ordering follow the
/// resolver-owned platform comparison; on Windows that conservatively folds case aliases so a
/// sorted admission set cannot treat them as independent writers. This type deliberately does not
/// implement `Hash`: callers that need a resolved identity set should preserve the single ordering
/// contract instead of inventing a second platform key representation.
#[derive(Clone, Debug)]
pub struct ResolvedProjectPathIdentity {
    resolved: ResolvedProjectPath,
}

impl ResolvedProjectPathIdentity {
    pub fn operation_path(&self) -> &Path {
        self.resolved.operation_path()
    }

    /// Returns whether this identity is equal to or contained by `root`.
    pub fn is_within(&self, root: &Self) -> bool {
        self.relative_to(root).is_some()
    }

    pub(crate) fn relative_to(&self, root: &Self) -> Option<std::path::PathBuf> {
        #[cfg(windows)]
        {
            super::windows::strip_path_prefix_ignore_case(
                self.operation_path(),
                root.operation_path(),
            )
        }
        #[cfg(not(windows))]
        {
            self.operation_path()
                .strip_prefix(root.operation_path())
                .ok()
                .map(Path::to_path_buf)
        }
    }
}

impl From<ResolvedProjectPath> for ResolvedProjectPathIdentity {
    fn from(resolved: ResolvedProjectPath) -> Self {
        Self { resolved }
    }
}

impl PartialEq for ResolvedProjectPathIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for ResolvedProjectPathIdentity {}

impl PartialOrd for ResolvedProjectPathIdentity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ResolvedProjectPathIdentity {
    fn cmp(&self, other: &Self) -> Ordering {
        #[cfg(windows)]
        {
            super::windows::compare_paths_ignore_case(self.operation_path(), other.operation_path())
        }
        #[cfg(not(windows))]
        {
            self.operation_path().cmp(other.operation_path())
        }
    }
}
