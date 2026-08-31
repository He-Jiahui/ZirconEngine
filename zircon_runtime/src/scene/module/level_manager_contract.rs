use std::path::Path;
use std::sync::Arc;

use crate::asset::project::ProjectManager;
use crate::asset::{AssetManager, asset_manager_handle};
use crate::core::CoreError;
use crate::core::framework::scene::{
    LevelManager as LevelManagerContract, LevelManagerError, LevelSummary, SceneArtifactTicket,
    WorldHandle,
};
use crate::core::manager::resolve_manager_service;
use crate::core::resource::ResourceLocator;

use super::DefaultLevelManager;

impl LevelManagerContract for DefaultLevelManager {
    fn create_default_level_handle(&self) -> Result<WorldHandle, LevelManagerError> {
        self.try_create_default_level()
            .map(|level| level.handle())
            .map_err(map_create_error)
    }

    fn level_exists(&self, handle: WorldHandle) -> bool {
        self.lock_levels().contains_key(&handle)
    }

    fn level_summary(&self, handle: WorldHandle) -> Option<LevelSummary> {
        self.level(handle).map(|level| {
            level.with_world(|world| LevelSummary {
                handle,
                entity_count: world.nodes().len(),
                active_camera: Some(world.active_camera()),
            })
        })
    }

    fn load_level_asset(
        &self,
        project_root: &str,
        uri: &str,
    ) -> Result<WorldHandle, LevelManagerError> {
        let project = self.active_project_snapshot(project_root)?;
        let uri = ResourceLocator::parse(uri).map_err(|error| {
            LevelManagerError::InvalidResourceLocator {
                uri: uri.to_string(),
                reason: error.to_string(),
            }
        })?;
        self.load_level(&project, &uri)
            .map(|level| level.handle())
            .map_err(|error| LevelManagerError::LoadFailed {
                uri: uri.to_string(),
                reason: error.to_string(),
            })
    }

    fn save_level_asset(
        &self,
        handle: WorldHandle,
        project_root: &str,
        uri: &str,
    ) -> Result<Arc<dyn SceneArtifactTicket>, LevelManagerError> {
        let project = self.active_project_snapshot(project_root)?;
        let uri = ResourceLocator::parse(uri).map_err(|error| {
            LevelManagerError::InvalidResourceLocator {
                uri: uri.to_string(),
                reason: error.to_string(),
            }
        })?;
        self.save_level(handle, &project, &uri)
            .map_err(|error| LevelManagerError::SaveFailed {
                handle,
                uri: uri.to_string(),
                reason: error.to_string(),
            })
    }
}

impl DefaultLevelManager {
    fn active_project_snapshot(
        &self,
        expected_root: &str,
    ) -> Result<ProjectManager, LevelManagerError> {
        let core = self
            .core
            .as_ref()
            .and_then(crate::core::CoreWeak::upgrade)
            .ok_or(LevelManagerError::RuntimeUnavailable)?;
        let asset_manager = asset_manager_handle(&core)
            .and_then(|handle| resolve_manager_service(&core, handle))
            .map_err(|error| LevelManagerError::AssetManagerUnavailable {
                reason: error.to_string(),
            })?;
        let project = asset_manager
            .current_project_snapshot()
            .ok_or(LevelManagerError::ProjectUnavailable)?;
        if project.paths().root() != Path::new(expected_root) {
            return Err(LevelManagerError::ProjectRootMismatch {
                active: project.paths().root().to_string_lossy().into_owned(),
                requested: expected_root.to_string(),
            });
        }
        Ok(project)
    }
}

fn map_create_error(error: CoreError) -> LevelManagerError {
    match error {
        CoreError::LevelHandleExhausted => LevelManagerError::HandleSpaceExhausted,
        CoreError::RuntimeUnavailable => LevelManagerError::RuntimeUnavailable,
        error => LevelManagerError::CreateFailed {
            reason: error.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use crate::core::framework::scene::{LevelManager, LevelManagerError};

    use super::DefaultLevelManager;

    const CONTRACT_SOURCE: &str = include_str!("level_manager_contract.rs");

    #[test]
    fn level_manager_asset_io_uses_the_active_project_generation_without_a_scan() {
        assert!(CONTRACT_SOURCE.contains(concat!("current_project_", "snapshot()")));
        assert!(!CONTRACT_SOURCE.contains(concat!("ProjectManager", "::open")));
        assert!(!CONTRACT_SOURCE.contains(concat!("scan_and_", "import")));
    }

    #[test]
    fn level_manager_contract_maps_kernel_handle_exhaustion_to_its_domain_error() {
        let manager = DefaultLevelManager::default();
        manager.next_handle.store(u64::MAX, Ordering::Relaxed);

        assert_eq!(
            LevelManager::create_default_level_handle(&manager),
            Err(LevelManagerError::HandleSpaceExhausted)
        );
    }
}
