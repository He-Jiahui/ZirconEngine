//! Scene module registration and level manager services.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use zircon_asset::ProjectManager;
use zircon_core::{
    CoreError, CoreHandle, DriverDescriptor, ManagerDescriptor, ModuleDescriptor, ServiceKind,
    ServiceObject, StartupMode,
};
use zircon_manager::{
    LevelManager as LevelManagerFacade, LevelManagerHandle, LevelSummary, WorldHandle,
};
use zircon_module::{dependency_on, factory, qualified_name};
use zircon_resource::ResourceLocator;

use crate::level_system::{LevelMetadata, LevelSystem};
use crate::serializer::SceneAssetSerializer;
use crate::world::{SceneProjectError, World};

pub const SCENE_MODULE_NAME: &str = "SceneModule";
pub const WORLD_DRIVER_NAME: &str = "SceneModule.Driver.WorldDriver";
pub const DEFAULT_LEVEL_MANAGER_NAME: &str = "SceneModule.Manager.DefaultLevelManager";
pub const LEVEL_MANAGER_NAME: &str = zircon_manager::LEVEL_MANAGER_NAME;

#[derive(Debug, Default)]
pub struct WorldDriver;

#[derive(Debug, Default)]
pub struct DefaultLevelManager {
    next_handle: AtomicU64,
    levels: Mutex<HashMap<WorldHandle, LevelSystem>>,
}

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
                selected_entity: world.selected_node(),
                active_camera: Some(world.active_camera()),
            })
        })
    }

    fn load_level_asset(&self, project_root: &str, uri: &str) -> Result<WorldHandle, CoreError> {
        let mut project =
            ProjectManager::open(project_root).map_err(|error| scene_core_error(error.to_string()))?;
        project
            .scan_and_import()
            .map_err(|error| scene_core_error(error.to_string()))?;
        let uri = ResourceLocator::parse(uri).map_err(|error| scene_core_error(error.to_string()))?;
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
        let mut project =
            ProjectManager::open(project_root).map_err(|error| scene_core_error(error.to_string()))?;
        project
            .scan_and_import()
            .map_err(|error| scene_core_error(error.to_string()))?;
        let uri = ResourceLocator::parse(uri).map_err(|error| scene_core_error(error.to_string()))?;
        self.save_level(handle, &project, &uri)
            .map_err(|error| scene_core_error(error.to_string()))
    }
}

impl DefaultLevelManager {
    pub fn create_default_level(&self) -> LevelSystem {
        self.create_level(World::new(), LevelMetadata::default())
    }

    pub fn create_level(&self, world: World, metadata: LevelMetadata) -> LevelSystem {
        let handle = WorldHandle::new(self.next_handle.fetch_add(1, Ordering::SeqCst) + 1);
        let level = LevelSystem::new(handle, Arc::new(Mutex::new(world)), metadata);
        self.levels.lock().unwrap().insert(handle, level.clone());
        level
    }

    pub fn level(&self, handle: WorldHandle) -> Option<LevelSystem> {
        self.levels.lock().unwrap().get(&handle).cloned()
    }

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
        fs::write(path, scene.to_toml_string().map_err(|error| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string())
        })?)?;
        Ok(())
    }
}

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
    let mut project = ProjectManager::open(project_root)
        .map_err(|error| scene_core_error(error.to_string()))?;
    project
        .scan_and_import()
        .map_err(|error| scene_core_error(error.to_string()))?;
    let uri = ResourceLocator::parse(uri).map_err(|error| scene_core_error(error.to_string()))?;
    manager
        .load_level(&project, &uri)
        .map_err(|error| scene_core_error(error.to_string()))
}

fn scene_core_error(message: impl Into<String>) -> CoreError {
    CoreError::Initialization(LEVEL_MANAGER_NAME.to_string(), message.into())
}

fn display_name_for_level(uri: &ResourceLocator) -> Option<String> {
    let source = uri.label().unwrap_or(uri.path());
    source.rsplit('/').next().map(ToString::to_string)
}

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(
        SCENE_MODULE_NAME,
        "ECS world management, hierarchy, level lifecycle, and render extraction",
    )
    .with_driver(DriverDescriptor::new(
        qualified_name(SCENE_MODULE_NAME, ServiceKind::Driver, "WorldDriver"),
        StartupMode::Immediate,
        Vec::new(),
        factory(|_| Ok(Arc::new(WorldDriver) as ServiceObject)),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(SCENE_MODULE_NAME, ServiceKind::Manager, "DefaultLevelManager"),
        StartupMode::Immediate,
        Vec::new(),
        factory(|_| Ok(Arc::new(DefaultLevelManager::default()) as ServiceObject)),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(SCENE_MODULE_NAME, ServiceKind::Manager, "LevelManager"),
        StartupMode::Immediate,
        vec![dependency_on(
            SCENE_MODULE_NAME,
            ServiceKind::Manager,
            "DefaultLevelManager",
        )],
        factory(|core| {
            let manager = resolve_default_level_manager(core)?;
            Ok(Arc::new(LevelManagerHandle::new(manager)) as ServiceObject)
        }),
    ))
}

#[cfg(test)]
mod tests {
    use zircon_resource::ResourceLocator;

    use super::display_name_for_level;

    #[test]
    fn display_name_prefers_label_when_present() {
        let uri = ResourceLocator::parse("res://scenes/main.scene.toml#CameraPreview").unwrap();
        assert_eq!(
            display_name_for_level(&uri).as_deref(),
            Some("CameraPreview")
        );
    }

    #[test]
    fn display_name_falls_back_to_last_path_segment() {
        let uri = ResourceLocator::parse("res://scenes/main.scene.toml").unwrap();
        assert_eq!(
            display_name_for_level(&uri).as_deref(),
            Some("main.scene.toml")
        );
    }
}
