use std::sync::Arc;

use zircon_runtime::asset::{
    project_asset_manager_handle, ProjectAssetManagerAccess, ASSET_MODULE_NAME,
};
use zircon_runtime::core::framework::navigation::NavigationManager;
use zircon_runtime::core::runtime::{ServiceObject, TASKS_MODULE_NAME};
use zircon_runtime::core::{
    CoreError, DriverDescriptor, ManagerDescriptor, ModuleDependencySpec, ModuleDescriptor,
    ServiceKind, StartupMode,
};
use zircon_runtime::engine_module::{dependency_on, factory, qualified_name};
use zircon_runtime::scene::{SceneNavigationRuntime, SCENE_NAVIGATION_RUNTIME_DRIVER_NAME};

#[path = "agent.rs"]
mod agent;
#[path = "capability.rs"]
mod capability;
#[path = "component_json.rs"]
mod component_json;
#[path = "components.rs"]
mod components;
#[path = "manager.rs"]
mod manager;
#[path = "off_mesh_connections.rs"]
mod off_mesh_connections;
#[path = "overlay_frame.rs"]
mod overlay_frame;
#[path = "plugin.rs"]
mod plugin;
#[path = "runtime_obstacles.rs"]
mod runtime_obstacles;
#[path = "settings_hash.rs"]
mod settings_hash;
#[path = "settings_validation.rs"]
mod settings_validation;

#[cfg(test)]
#[path = "tests/test_support.rs"]
mod test_support;

pub use capability::{
    NATIVE_PLUGIN_ID, NATIVE_REQUESTED_CAPABILITIES, NATIVE_RUNTIME_ENTRY,
    NATIVE_RUNTIME_REGISTRATION_MANIFEST, NAVIGATION_DECLARATION, NAVIGATION_RECAST_CAPABILITY,
    NAVIGATION_RUNTIME_CAPABILITY, PLUGIN_ID, RUNTIME_CAPABILITIES,
};
pub use components::navigation_component_descriptors;
pub use manager::{
    count_navigation_components, default_agent_type, DefaultNavigationManager,
    NavMeshBakeTaskHandle, NavMeshBakeTaskState, NavMeshDirtyBakeReport, NavMeshDirtyBounds,
};
pub use overlay_frame::{
    NavigationOverlayFrame, NAVIGATION_OVERLAY_FRAME_EVENT_ID,
    NAVIGATION_OVERLAY_FRAME_PAYLOAD_SCHEMA,
};
pub use plugin::{
    package_manifest, plugin_registration, runtime_capabilities, runtime_plugin,
    runtime_plugin_descriptor, NavigationRuntimePlugin, NAVIGATION_DIST_CRATE_NAME,
    NAVIGATION_DIST_RUNTIME_ENTRY, NAVIGATION_MAIN_SYSTEM_SET,
};

pub const NAVIGATION_MODULE_NAME: &str = "navigation.runtime";
pub const DEFAULT_NAVIGATION_RUNTIME_DRIVER_NAME: &str =
    "navigation.runtime.Driver.DefaultNavigationRuntime";
pub use zircon_runtime::core::manager::NAVIGATION_MANAGER_NAME;
pub const NAVIGATION_EVENT_NAMESPACE: &str = "navigation.events";

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(
        NAVIGATION_MODULE_NAME,
        "Navigation path query, bake, and agent runtime plugin",
    )
    .with_module_dependency(ModuleDependencySpec::named(TASKS_MODULE_NAME))
    .with_module_dependency(ModuleDependencySpec::named(ASSET_MODULE_NAME))
    .with_driver(DriverDescriptor::new(
        qualified_name(
            NAVIGATION_MODULE_NAME,
            ServiceKind::Driver,
            "DefaultNavigationRuntime",
        ),
        StartupMode::Lazy,
        vec![dependency_on(
            ASSET_MODULE_NAME,
            ServiceKind::Manager,
            "ProjectAssetManager",
        )],
        factory(|core| {
            let core = core.upgrade().ok_or(CoreError::RuntimeUnavailable)?;
            let project_assets =
                ProjectAssetManagerAccess::new(core.clone(), project_asset_manager_handle(&core)?);
            Ok(Arc::new(DefaultNavigationManager::new(
                core.task_graph().worker_pool().clone(),
                project_assets,
            )) as ServiceObject)
        }),
    ))
    .with_driver(DriverDescriptor::new(
        qualified_name(
            NAVIGATION_MODULE_NAME,
            ServiceKind::Driver,
            "SceneNavigationRuntime",
        ),
        StartupMode::Lazy,
        vec![dependency_on(
            NAVIGATION_MODULE_NAME,
            ServiceKind::Driver,
            "DefaultNavigationRuntime",
        )],
        factory(|core| {
            let core = core.upgrade().ok_or(CoreError::RuntimeUnavailable)?;
            core.register_service_alias::<DefaultNavigationManager, dyn SceneNavigationRuntime>(
                DEFAULT_NAVIGATION_RUNTIME_DRIVER_NAME,
                ServiceKind::Driver,
                SCENE_NAVIGATION_RUNTIME_DRIVER_NAME,
                ServiceKind::Driver,
                |manager| manager,
            )
        }),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(
            NAVIGATION_MODULE_NAME,
            ServiceKind::Manager,
            "NavigationManager",
        ),
        StartupMode::Lazy,
        vec![dependency_on(
            NAVIGATION_MODULE_NAME,
            ServiceKind::Driver,
            "DefaultNavigationRuntime",
        )],
        factory(|core| {
            let core = core.upgrade().ok_or(CoreError::RuntimeUnavailable)?;
            core.register_service_alias::<DefaultNavigationManager, dyn NavigationManager>(
                DEFAULT_NAVIGATION_RUNTIME_DRIVER_NAME,
                ServiceKind::Driver,
                NAVIGATION_MANAGER_NAME,
                ServiceKind::Manager,
                |manager| manager,
            )
        }),
    ))
}

pub fn navigation_plugin_options() -> Vec<zircon_runtime::plugin::PluginOptionManifest> {
    vec![
        zircon_runtime::plugin::PluginOptionManifest::new(
            "navigation.default_agent_type",
            "Default Agent Type",
            "string",
            default_agent_type(),
        ),
        zircon_runtime::plugin::PluginOptionManifest::new(
            "navigation.default_settings_asset",
            "Navigation Settings Asset",
            "string",
            "res://navigation/settings/default.navigation.toml",
        ),
        zircon_runtime::plugin::PluginOptionManifest::new(
            "navigation.debug_gizmos",
            "Navigation Debug Gizmos",
            "bool",
            "true",
        )
        .with_required_capability(NAVIGATION_RUNTIME_CAPABILITY),
        zircon_runtime::plugin::PluginOptionManifest::new(
            "navigation.bake_backend",
            "Navigation Bake Backend",
            "enum",
            "recast",
        )
        .with_enum_values(["recast"])
        .with_required_capability(NAVIGATION_RECAST_CAPABILITY),
    ]
}

pub fn navigation_event_catalog() -> zircon_runtime::plugin::PluginEventCatalogManifest {
    zircon_runtime::plugin::PluginEventCatalogManifest {
        namespace: NAVIGATION_EVENT_NAMESPACE.to_string(),
        version: 1,
        events: vec![
            zircon_runtime::plugin::PluginEventManifest {
                id: "navigation.events.navmesh_baked".to_string(),
                display_name: "NavMesh Baked".to_string(),
                payload_schema: "navigation.events.navmesh_bake_report.v1".to_string(),
            },
            zircon_runtime::plugin::PluginEventManifest {
                id: "navigation.events.path_query_completed".to_string(),
                display_name: "Path Query Completed".to_string(),
                payload_schema: "navigation.events.nav_path_result.v1".to_string(),
            },
            zircon_runtime::plugin::PluginEventManifest {
                id: "navigation.events.path_query_failed".to_string(),
                display_name: "Path Query Failed".to_string(),
                payload_schema: "navigation.events.navigation_error.v1".to_string(),
            },
            zircon_runtime::plugin::PluginEventManifest {
                id: "navigation.events.agent_tick_completed".to_string(),
                display_name: "Agent Tick Completed".to_string(),
                payload_schema: "navigation.events.nav_agent_tick_report.v1".to_string(),
            },
            zircon_runtime::plugin::PluginEventManifest {
                id: NAVIGATION_OVERLAY_FRAME_EVENT_ID.to_string(),
                display_name: "Navigation Overlay Frame".to_string(),
                payload_schema: NAVIGATION_OVERLAY_FRAME_PAYLOAD_SCHEMA.to_string(),
            },
            zircon_runtime::plugin::PluginEventManifest {
                id: "navigation.events.off_mesh_traverse".to_string(),
                display_name: "Off Mesh Traverse".to_string(),
                payload_schema: "navigation.events.off_mesh_traverse.v1".to_string(),
            },
        ],
    }
}

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
