use std::sync::Arc;

use zircon_runtime::core::manager::NavigationManagerHandle;
use zircon_runtime::core::runtime::ServiceObject;
use zircon_runtime::core::{ManagerDescriptor, ModuleDescriptor, ServiceKind, StartupMode};
use zircon_runtime::engine_module::{factory, qualified_name};

mod capability;
mod component_json;
mod components;
mod manager;
mod off_mesh_connections;
mod runtime_obstacles;
mod settings_hash;
mod settings_validation;

pub use capability::{
    NAVIGATION_RECAST_CAPABILITY, NAVIGATION_RUNTIME_CAPABILITY, RUNTIME_CAPABILITIES,
};
pub use components::navigation_component_descriptors;
pub use manager::{count_navigation_components, default_agent_type, DefaultNavigationManager};

pub const PLUGIN_ID: &str = "navigation";
pub const NAVIGATION_MODULE_NAME: &str = "NavigationModule";
pub use zircon_runtime::core::manager::NAVIGATION_MANAGER_NAME;
pub const NAVIGATION_EVENT_NAMESPACE: &str = "navigation.runtime";

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(
        NAVIGATION_MODULE_NAME,
        "Navigation path query, bake, and agent runtime plugin",
    )
    .with_manager(ManagerDescriptor::new(
        qualified_name(
            NAVIGATION_MODULE_NAME,
            ServiceKind::Manager,
            "NavigationManager",
        ),
        StartupMode::Lazy,
        Vec::new(),
        factory(|_| {
            Ok(Arc::new(NavigationManagerHandle::new(Arc::new(
                DefaultNavigationManager::new(),
            ))) as ServiceObject)
        }),
    ))
}

#[derive(Clone, Debug)]
pub struct NavigationRuntimePlugin {
    descriptor: zircon_runtime::plugin::RuntimePluginDescriptor,
}

impl NavigationRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl Default for NavigationRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl zircon_runtime::plugin::RuntimePlugin for NavigationRuntimePlugin {
    fn descriptor(&self) -> &zircon_runtime::plugin::RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> zircon_runtime::plugin::PluginPackageManifest {
        let mut manifest = self.descriptor.package_manifest();
        for descriptor in navigation_component_descriptors() {
            manifest = manifest.with_component(descriptor);
        }
        for option in navigation_plugin_options() {
            manifest = manifest.with_option(option);
        }
        manifest.with_event_catalog(navigation_event_catalog())
    }

    fn register(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        registry.register_module(module_descriptor())?;
        for descriptor in navigation_component_descriptors() {
            registry.register_component(descriptor)?;
        }
        for option in navigation_plugin_options() {
            registry.register_plugin_option(option)?;
        }
        registry.register_plugin_event_catalog(navigation_event_catalog())?;
        Ok(())
    }
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
                id: "navigation.runtime.navmesh_baked".to_string(),
                display_name: "NavMesh Baked".to_string(),
                payload_schema: "navigation.runtime.navmesh_bake_report.v1".to_string(),
            },
            zircon_runtime::plugin::PluginEventManifest {
                id: "navigation.runtime.path_query_completed".to_string(),
                display_name: "Path Query Completed".to_string(),
                payload_schema: "navigation.runtime.nav_path_result.v1".to_string(),
            },
            zircon_runtime::plugin::PluginEventManifest {
                id: "navigation.runtime.path_query_failed".to_string(),
                display_name: "Path Query Failed".to_string(),
                payload_schema: "navigation.runtime.navigation_error.v1".to_string(),
            },
            zircon_runtime::plugin::PluginEventManifest {
                id: "navigation.runtime.agent_tick_completed".to_string(),
                display_name: "Agent Tick Completed".to_string(),
                payload_schema: "navigation.runtime.nav_agent_tick_report.v1".to_string(),
            },
        ],
    }
}

pub fn runtime_plugin_descriptor() -> zircon_runtime::plugin::RuntimePluginDescriptor {
    zircon_runtime::plugin::RuntimePluginDescriptor::builder(
        PLUGIN_ID,
        "Navigation",
        zircon_runtime::builtin::RuntimePluginId::Navigation,
        "zircon_plugin_navigation_runtime",
    )
    .with_target_modes([
        zircon_runtime::builtin::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::builtin::RuntimeTargetMode::ServerRuntime,
        zircon_runtime::builtin::RuntimeTargetMode::EditorHost,
    ])
    .with_capability(NAVIGATION_RUNTIME_CAPABILITY)
    .with_capability(NAVIGATION_RECAST_CAPABILITY)
    .with_maturity(zircon_runtime::plugin::PluginMaturity::Beta)
    .with_capability_status(
        zircon_runtime::plugin::CapabilityStatusManifest::new(
            NAVIGATION_RUNTIME_CAPABILITY,
            zircon_runtime::plugin::CapabilityStatus::Partial,
        )
        .with_note("Gameplay navmesh/pathfinding is optional; UI navigation parity is separate."),
    )
    .build()
}

zircon_plugin_sdk::runtime_plugin_exports!(NavigationRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    RUNTIME_CAPABILITIES
}

#[cfg(test)]
mod tests;
