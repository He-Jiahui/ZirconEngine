use zircon_asset::ProjectManager;
use zircon_core::CoreError;
use zircon_framework::scene::LevelManager as LevelManagerFacade;
use zircon_resource::ResourceLocator;

use super::core_error::scene_core_error;
use super::DefaultLevelManager;
use crate::{LevelSummary, WorldHandle};

impl LevelManagerFacade for DefaultLevelManager {
    fn create_default_level_handle(&self) -> WorldHandle {
        self.create_default_level().handle()
    }

    fn level_exists(&self, handle: WorldHandle) -> bool {
        self.level(handle).is_some()
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
        let mut project = ProjectManager::open(project_root)
            .map_err(|error| scene_core_error(error.to_string()))?;
        project
            .scan_and_import()
            .map_err(|error| scene_core_error(error.to_string()))?;
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
    ) -> Result<(), CoreError> {
        let mut project = ProjectManager::open(project_root)
            .map_err(|error| scene_core_error(error.to_string()))?;
        project
            .scan_and_import()
            .map_err(|error| scene_core_error(error.to_string()))?;
        let uri =
            ResourceLocator::parse(uri).map_err(|error| scene_core_error(error.to_string()))?;
        self.save_level(handle, &project, &uri)
            .map_err(|error| scene_core_error(error.to_string()))
    }
}
