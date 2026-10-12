use std::fmt;

use super::ProjectAssetManager;

impl fmt::Debug for ProjectAssetManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProjectAssetManager")
            .field("worker_task_pool", &self.worker_task_pool)
            .finish_non_exhaustive()
    }
}
