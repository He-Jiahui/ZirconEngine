use std::sync::Arc;

use zircon_runtime::core::framework::navigation::NavigationManager;
use zircon_runtime::core::manager::RegisteredManagerService;
use zircon_runtime::core::runtime::ServiceObject;
use zircon_runtime::core::{
    DriverDescriptor, ManagerDescriptor, ModuleDescriptor, ServiceKind, StartupMode,
};
use zircon_runtime::engine_module::{dependency_on, factory, qualified_name};
use zircon_runtime::scene::SceneNavigationRuntimeHandle;

mod agent;
mod capability;
mod component_json;
mod components;
mod manager;
mod off_mesh_connections;
mod plugin;
mod runtime_obstacles;
mod settings_hash;
mod settings_validation;

pub use capability::{
    NAVIGATION_RECAST_CAPABILITY, NAVIGATION_RUNTIME_CAPABILITY, RUNTIME_CAPABILITIES,
};
pub use components::navigation_component_descriptors;
pub use manager::{
    count_navigation_components, default_agent_type, DefaultNavigationManager,
    NavMeshBakeTaskHandle, NavMeshBakeTaskState, NavMeshDirtyBakeReport, NavMeshDirtyBounds,
};
pub use plugin::{
    package_manifest, plugin_registration, runtime_capabilities, runtime_plugin,
    runtime_plugin_descriptor, NavigationRuntimePlugin, NAVIGATION_DIST_CRATE_NAME,
    NAVIGATION_DIST_RUNTIME_ENTRY,
};

pub const PLUGIN_ID: &str = "navigation";
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
    .with_driver(DriverDescriptor::new(
        qualified_name(
            NAVIGATION_MODULE_NAME,
            ServiceKind::Driver,
            "DefaultNavigationRuntime",
        ),
        StartupMode::Lazy,
        Vec::new(),
        factory(|_| Ok(Arc::new(DefaultNavigationManager::new()) as ServiceObject)),
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
            let manager = core.resolve_driver::<DefaultNavigationManager>(
                DEFAULT_NAVIGATION_RUNTIME_DRIVER_NAME,
            )?;
            Ok(Arc::new(SceneNavigationRuntimeHandle::new(manager)) as ServiceObject)
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
            let manager = core.resolve_driver::<DefaultNavigationManager>(
                DEFAULT_NAVIGATION_RUNTIME_DRIVER_NAME,
            )?;
            Ok(
                Arc::new(RegisteredManagerService::<dyn NavigationManager>::new(
                    manager,
                )) as ServiceObject,
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
                id: "navigation.events.off_mesh_traverse".to_string(),
                display_name: "Off Mesh Traverse".to_string(),
                payload_schema: "navigation.events.off_mesh_traverse.v1".to_string(),
            },
        ],
    }
}

#[cfg(test)]
mod tests;
