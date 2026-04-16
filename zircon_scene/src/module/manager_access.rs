use std::sync::Arc;

use zircon_asset::ProjectManager;
use zircon_core::{CoreError, CoreHandle};
use zircon_resource::ResourceLocator;

use super::core_error::scene_core_error;
use super::{DefaultLevelManager, DEFAULT_LEVEL_MANAGER_NAME};
use crate::LevelSystem;

pub(crate) fn resolve_default_level_manager(
    core: &CoreHandle,
) -> Result<Arc<DefaultLevelManager>, CoreError> {
    core.resolve_manager::<DefaultLevelManager>(DEFAULT_LEVEL_MANAGER_NAME)
}

pub fn create_default_level(core: &CoreHandle) -> Result<LevelSystem, CoreError> {
    resolve_default_level_manager(core).map(|manager| manager.create_default_level())
}

pub fn load_level_asset(
    core: &CoreHandle,
    project_root: &str,
    uri: &str,
) -> Result<LevelSystem, CoreError> {
    let manager = resolve_default_level_manager(core)?;
    let mut project =
        ProjectManager::open(project_root).map_err(|error| scene_core_error(error.to_string()))?;
    project
        .scan_and_import()
        .map_err(|error| scene_core_error(error.to_string()))?;
    let uri = ResourceLocator::parse(uri).map_err(|error| scene_core_error(error.to_string()))?;
    manager
        .load_level(&project, &uri)
        .map_err(|error| scene_core_error(error.to_string()))
}
