use zircon_plugin_sdk::{TestRuntime, WeakBridge};
use zircon_runtime::asset::{self, ProjectAssetManager};
use zircon_runtime::core::CoreHandle;
use zircon_runtime::plugin::RuntimeExtensionCatalogReport;

use zircon_plugin_physics_runtime::PhysicsQueryInterface;

pub(super) fn runtime_with_physics_animation_scene_asset() -> TestRuntime {
    let physics_plugin = zircon_plugin_physics_runtime::runtime_plugin();
    let animation_plugin = zircon_plugin_animation_runtime::runtime_plugin();
    TestRuntime::builder()
        .with_runtime_plugin(&physics_plugin)
        .with_runtime_plugin(&animation_plugin)
        .build()
        .unwrap()
}

pub(super) fn runtime_physics_query_bridge(
    extension_report: &RuntimeExtensionCatalogReport,
) -> WeakBridge<dyn PhysicsQueryInterface> {
    extension_report
        .registry
        .frozen_bridge_table()
        .resolve_weak::<dyn PhysicsQueryInterface>()
}

pub(super) fn runtime_with_scene_asset_only() -> TestRuntime {
    TestRuntime::builder().build().unwrap()
}

pub(super) fn runtime_asset_manager(core: &CoreHandle) -> Arc<ProjectAssetManager> {
    core.resolve_manager::<ProjectAssetManager>(asset::PROJECT_ASSET_MANAGER_NAME)
        .unwrap()
}
