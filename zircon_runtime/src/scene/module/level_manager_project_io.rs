use std::path::Path;

use crate::asset::project::ProjectManager;
use crate::core::framework::scene::WorldHandle;
use crate::core::resource::ResourceLocator;

use super::level_display_name::display_name_for_level;
use super::DefaultLevelManager;
use crate::scene::{
    serializer::SceneAssetSerializer,
    world::{SceneProjectError, World},
    LevelMetadata, LevelSystem,
};

impl DefaultLevelManager {
    pub fn save_world(
        &self,
        handle: WorldHandle,
        path: impl AsRef<Path>,
    ) -> Result<(), SceneProjectError> {
        let level = self.level(handle).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "world handle not found")
        })?;
        level.snapshot().save_project_to_path(path)
    }

    pub fn load_world(&self, path: impl AsRef<Path>) -> Result<LevelSystem, SceneProjectError> {
        let world = World::load_project_from_path(path)?;
        self.try_create_level(world, LevelMetadata::default())
            .map_err(|error| SceneProjectError::SceneAsset(error.to_string()))
    }

    pub fn load_level(
        &self,
        project: &ProjectManager,
        uri: &ResourceLocator,
    ) -> Result<LevelSystem, SceneProjectError> {
        let world = SceneAssetSerializer::load_world(project, uri)?;
        self.try_create_level(
            world,
            LevelMetadata {
                project_root: Some(project.paths().root().to_string_lossy().into_owned()),
                asset_uri: Some(uri.to_string()),
                display_name: display_name_for_level(uri),
            },
        )
        .map_err(|error| SceneProjectError::SceneAsset(error.to_string()))
    }

    pub fn save_level(
        &self,
        handle: WorldHandle,
        project: &ProjectManager,
        uri: &ResourceLocator,
    ) -> Result<(), SceneProjectError> {
        let level = self.level(handle).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "world handle not found")
        })?;
        level.snapshot().save_scene_to_project(project, uri)
    }
}
