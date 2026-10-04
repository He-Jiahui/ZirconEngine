use std::ops::Deref;

use zircon_runtime::asset::{
    module_descriptor as asset_module_descriptor, project_asset_manager_handle,
    ProjectAssetManagerAccess, ASSET_MODULE_NAME,
};
use zircon_runtime::core::runtime::{CoreRuntime, TasksModule};
use zircon_runtime::engine_module::EngineModule;
use zircon_runtime::scene::{DefaultLevelManager, LevelMetadata, LevelSystem, World};

use crate::DefaultNavigationManager;

#[derive(Clone)]
pub(crate) struct TestNavigationManager {
    manager: DefaultNavigationManager,
    _runtime: CoreRuntime,
}

impl Deref for TestNavigationManager {
    type Target = DefaultNavigationManager;

    fn deref(&self) -> &Self::Target {
        &self.manager
    }
}

pub(crate) fn navigation_manager() -> TestNavigationManager {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(TasksModule.descriptor())
        .expect("navigation test tasks module");
    runtime
        .register_module(asset_module_descriptor())
        .expect("navigation test asset module");
    runtime
        .activate_module(ASSET_MODULE_NAME)
        .expect("navigation test asset module activation");
    let core = runtime.handle();
    let project_assets = ProjectAssetManagerAccess::new(
        core.clone(),
        project_asset_manager_handle(&core).expect("navigation test project asset handle"),
    );
    let manager =
        DefaultNavigationManager::new(runtime.task_graph().worker_pool().clone(), project_assets);
    TestNavigationManager {
        manager,
        _runtime: runtime,
    }
}

pub(crate) fn level_from_world(world: World) -> (DefaultLevelManager, LevelSystem) {
    let owner = DefaultLevelManager::default();
    let level = owner.create_level(world, LevelMetadata::default());
    (owner, level)
}
