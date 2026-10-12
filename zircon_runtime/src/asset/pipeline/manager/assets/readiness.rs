#[cfg(test)]
use std::sync::RwLockWriteGuard;
use std::sync::TryLockError;

use super::ProjectAssetManager;

impl ProjectAssetManager {
    pub(in crate::asset) fn catalog_generation_is_ready(&self) -> bool {
        match self.project_generation_gate.try_read() {
            Ok(_) | Err(TryLockError::Poisoned(_)) => true,
            Err(TryLockError::WouldBlock) => false,
        }
    }

    #[cfg(test)]
    pub(in crate::asset) fn hold_catalog_generation_publication_for_test(
        &self,
    ) -> RwLockWriteGuard<'_, ()> {
        self.project_generation_write()
    }
}
