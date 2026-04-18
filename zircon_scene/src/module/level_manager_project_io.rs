use std::fs;
use std::path::Path;

use zircon_asset::ProjectManager;
use zircon_resource::ResourceLocator;

use super::level_display_name::display_name_for_level;
use super::DefaultLevelManager;
use crate::{
    LevelMetadata, LevelSystem, SceneAssetSerializer, SceneProjectError, World, WorldHandle,
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
        Ok(self.create_level(world, LevelMetadata::default()))
    }

    pub fn load_level(
        &self,
        project: &ProjectManager,
        uri: &ResourceLocator,
    ) -> Result<LevelSystem, SceneProjectError> {
        let world = SceneAssetSerializer::load_world(project, uri)?;
        Ok(self.create_level(
            world,
            LevelMetadata {
                project_root: Some(project.paths().root().to_string_lossy().into_owned()),
                asset_uri: Some(uri.to_string()),
                display_name: display_name_for_level(uri),
            },
        ))
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
        let scene = SceneAssetSerializer::serialize_world(project, &level.snapshot())?;
        let path = project.source_path_for_uri(uri)?;
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(
            path,
            scene.to_toml_string().map_err(|error| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string())
            })?,
        )?;
        Ok(())
    }
}
