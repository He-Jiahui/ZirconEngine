use std::path::Path;
use std::sync::Arc;

use crate::asset::project::ProjectManager;
use crate::asset::{asset_manager_handle, AssetManager};
use crate::core::framework::scene::{
    LevelManager as LevelManagerContract, LevelSummary, SceneArtifactTicket, WorldHandle,
};
use crate::core::manager::resolve_manager_service;
use crate::core::resource::ResourceLocator;
use crate::core::CoreError;

use super::core_error::scene_core_error;
use super::DefaultLevelManager;

impl LevelManagerContract for DefaultLevelManager {
    fn create_default_level_handle(&self) -> Result<WorldHandle, CoreError> {
        self.try_create_default_level().map(|level| level.handle())
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

    fn load_level_asset(&self, project_root: &str, uri: &str) -> Result<WorldHandle, CoreError> {
        let project = self.active_project_snapshot(project_root)?;
        let uri =
            ResourceLocator::parse(uri).map_err(|error| scene_core_error(error.to_string()))?;
        self.load_level(&project, &uri)
            .map(|level| level.handle())
            .map_err(|error| scene_core_error(error.to_string()))
    }

    fn save_level_asset(
        &self,
        handle: WorldHandle,
        project_root: &str,
        uri: &str,
    ) -> Result<Arc<dyn SceneArtifactTicket>, CoreError> {
        let project = self.active_project_snapshot(project_root)?;
        let uri =
            ResourceLocator::parse(uri).map_err(|error| scene_core_error(error.to_string()))?;
        self.save_level(handle, &project, &uri)
            .map_err(|error| scene_core_error(error.to_string()))
    }
}

impl DefaultLevelManager {
    fn active_project_snapshot(&self, expected_root: &str) -> Result<ProjectManager, CoreError> {
        let core = self
            .core
            .as_ref()
            .and_then(crate::core::CoreWeak::upgrade)
            .ok_or_else(|| scene_core_error("LevelManager has no active Core runtime"))?;
        let asset_manager = asset_manager_handle(&core)
            .and_then(|handle| resolve_manager_service(&core, handle))?;
        let project = asset_manager
            .current_project_snapshot()
            .ok_or_else(|| scene_core_error("AssetManager has no active project generation"))?;
        if project.paths().root() != Path::new(expected_root) {
            return Err(scene_core_error(format!(
                "active project root {} does not match requested root {}",
                project.paths().root().display(),
                expected_root
            )));
        }
        Ok(project)
    }
}

#[cfg(test)]
mod tests {
    const CONTRACT_SOURCE: &str = include_str!("level_manager_contract.rs");

    #[test]
    fn level_manager_asset_io_uses_the_active_project_generation_without_a_scan() {
        assert!(CONTRACT_SOURCE.contains(concat!("current_project_", "snapshot()")));
        assert!(!CONTRACT_SOURCE.contains(concat!("ProjectManager", "::open")));
        assert!(!CONTRACT_SOURCE.contains(concat!("scan_and_", "import")));
    }
}
