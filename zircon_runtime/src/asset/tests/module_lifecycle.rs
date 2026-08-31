use std::sync::Arc;

use crate::asset::pipeline::manager::{project_asset_manager_handle, ProjectAssetManager};
use crate::asset::{module_descriptor, ASSET_MODULE_NAME};
use crate::core::manager::{resolve_manager_service, RegisteredManagerService};
use crate::core::runtime::ServiceObject;
use crate::core::{
    CoreRuntime, ManagerDescriptor, ModuleContext, ModuleDescriptor, RegistryName, ServiceKind,
    StartupMode,
};

#[test]
fn asset_module_manager_uses_the_activating_runtime_io_owner() {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(module_descriptor())
        .expect("asset module should register");
    runtime
        .activate_module(ASSET_MODULE_NAME)
        .expect("asset module should activate");

    let core = runtime.handle();
    let manager: Arc<ProjectAssetManager> =
        resolve_manager_service(&core, project_asset_manager_handle(&core).unwrap()).unwrap();

    assert!(manager
        .worker_task_pool()
        .shares_execution_owner_with(runtime.task_graph().worker_pool()));
}

#[test]
fn asset_module_readiness_tracks_project_catalog_generation_publication() {
    let runtime = CoreRuntime::new();
    let project_assets = Arc::new(ProjectAssetManager::default());
    let registered_project_assets = Arc::clone(&project_assets);
    runtime
        .register_module(
            ModuleDescriptor::new(ASSET_MODULE_NAME, "asset readiness integration probe")
                .with_manager(ManagerDescriptor::new(
                    RegistryName::from_parts(
                        ASSET_MODULE_NAME,
                        ServiceKind::Manager,
                        "ProjectAssetManager",
                    ),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(move |_| {
                        Ok(Arc::new(RegisteredManagerService::new(Arc::clone(
                            &registered_project_assets,
                        ))) as ServiceObject)
                    }),
                )),
        )
        .expect("asset readiness probe should register");
    runtime
        .activate_module(ASSET_MODULE_NAME)
        .expect("asset readiness probe manager should activate");

    let lifecycle = module_descriptor().lifecycle;
    let context = ModuleContext {
        module_name: ASSET_MODULE_NAME.to_owned(),
        core: runtime.handle().downgrade(),
    };
    let generation_publication = project_assets.hold_catalog_generation_publication_for_test();

    assert!(!lifecycle.ready(&context).expect("readiness should resolve"));
    drop(generation_publication);
    assert!(lifecycle.ready(&context).expect("readiness should resolve"));
}
