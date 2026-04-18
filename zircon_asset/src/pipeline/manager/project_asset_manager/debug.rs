use std::fmt;

use super::ProjectAssetManager;

impl fmt::Debug for ProjectAssetManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProjectAssetManager")
            .field("default_worker_count", &self.default_worker_count)
            .finish_non_exhaustive()
    }
}
