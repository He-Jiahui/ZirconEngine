use std::sync::Arc;

use zircon_runtime::core::{
    ManagerDescriptor, ModuleDescriptor, ServiceKind, ServiceObject, StartupMode,
};
use zircon_runtime::engine_module::{factory, qualified_name};

mod components;
mod manager;

pub use components::navigation_component_descriptors;
pub use manager::{count_navigation_components, default_agent_type, DefaultNavigationManager};

pub const PLUGIN_ID: &str = "navigation";
pub const NAVIGATION_MODULE_NAME: &str = "NavigationModule";
pub const NAVIGATION_MANAGER_NAME: &str = "NavigationModule.Manager.NavigationManager";
pub const NAVIGATION_EVENT_NAMESPACE: &str = "navigation";

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
        factory(|_| Ok(Arc::new(DefaultNavigationManager::new()) as ServiceObject)),
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

    fn register_runtime_extensions(
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
            "resource.NavigationSettings",
            "",
        ),
        zircon_runtime::plugin::PluginOptionManifest::new(
            "navigation.debug_gizmos",
            "Navigation Debug Gizmos",
            "bool",
            "true",
        )
        .with_required_capability("runtime.plugin.navigation"),
        zircon_runtime::plugin::PluginOptionManifest::new(
            "navigation.bake_backend",
            "Navigation Bake Backend",
            "enum.recast",
            "recast",
        )
        .with_required_capability("runtime.plugin.navigation.recast"),
    ]
}

pub fn navigation_event_catalog() -> zircon_runtime::plugin::PluginEventCatalogManifest {
    zircon_runtime::plugin::PluginEventCatalogManifest {
        namespace: NAVIGATION_EVENT_NAMESPACE.to_string(),
        version: 1,
        events: vec![
            zircon_runtime::plugin::PluginEventManifest {
                id: "navmesh_baked".to_string(),
                display_name: "NavMesh Baked".to_string(),
                payload_schema: "navigation.NavMeshBakeReport".to_string(),
            },
            zircon_runtime::plugin::PluginEventManifest {
                id: "path_query_completed".to_string(),
                display_name: "Path Query Completed".to_string(),
                payload_schema: "navigation.NavPathResult".to_string(),
            },
            zircon_runtime::plugin::PluginEventManifest {
                id: "path_query_failed".to_string(),
                display_name: "Path Query Failed".to_string(),
                payload_schema: "navigation.NavigationError".to_string(),
            },
            zircon_runtime::plugin::PluginEventManifest {
                id: "agent_tick_completed".to_string(),
                display_name: "Agent Tick Completed".to_string(),
                payload_schema: "navigation.NavAgentTickReport".to_string(),
            },
        ],
    }
}

pub fn runtime_plugin_descriptor() -> zircon_runtime::plugin::RuntimePluginDescriptor {
    zircon_runtime::plugin::RuntimePluginDescriptor::new(
        PLUGIN_ID,
        "Navigation",
        zircon_runtime::RuntimePluginId::Navigation,
        "zircon_plugin_navigation_runtime",
    )
    .with_target_modes([
        zircon_runtime::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::RuntimeTargetMode::ServerRuntime,
        zircon_runtime::RuntimeTargetMode::EditorHost,
    ])
    .with_capability("runtime.plugin.navigation")
    .with_capability("runtime.plugin.navigation.recast")
}

pub fn runtime_plugin() -> NavigationRuntimePlugin {
    NavigationRuntimePlugin::new()
}

pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    zircon_runtime::plugin::RuntimePlugin::package_manifest(&runtime_plugin())
}

pub fn runtime_selection() -> zircon_runtime::plugin::ProjectPluginSelection {
    zircon_runtime::plugin::RuntimePlugin::project_selection(&runtime_plugin())
}

pub fn plugin_registration() -> zircon_runtime::plugin::RuntimePluginRegistrationReport {
    zircon_runtime::plugin::RuntimePluginRegistrationReport::from_plugin(&runtime_plugin())
}

pub fn runtime_capabilities() -> &'static [&'static str] {
    &[
        "runtime.plugin.navigation",
        "runtime.plugin.navigation.recast",
    ]
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use zircon_runtime::asset::NavMeshAsset;
    use zircon_runtime::core::framework::navigation::{
        NavMeshAgentDescriptor, NavMeshBakeRequest, NavPathQuery, NavPathStatus, NavigationManager,
        AREA_JUMP, NAV_MESH_AGENT_COMPONENT_TYPE, NAV_MESH_MODIFIER_COMPONENT_TYPE,
        NAV_MESH_OBSTACLE_COMPONENT_TYPE, NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE,
        NAV_MESH_SURFACE_COMPONENT_TYPE,
    };
    use zircon_runtime::core::math::{Transform, Vec3};
    use zircon_runtime::core::CoreRuntime;
    use zircon_runtime::scene::components::NodeKind;
    use zircon_runtime::scene::world::World;

    use super::*;

    #[test]
    fn navigation_registration_contributes_runtime_module_and_components() {
        let report = plugin_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(report
            .extensions
            .modules()
            .iter()
            .any(|module| module.name == NAVIGATION_MODULE_NAME));
        for component_type in [
            NAV_MESH_SURFACE_COMPONENT_TYPE,
            NAV_MESH_MODIFIER_COMPONENT_TYPE,
            NAV_MESH_AGENT_COMPONENT_TYPE,
            NAV_MESH_OBSTACLE_COMPONENT_TYPE,
            NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE,
        ] {
            assert!(report
                .extensions
                .components()
                .iter()
                .any(|component| component.type_id == component_type));
        }
        assert!(report
            .extensions
            .plugin_options()
            .iter()
            .any(|option| option.key == "navigation.default_agent_type"));
        assert!(report
            .extensions
            .plugin_event_catalogs()
            .iter()
            .any(|catalog| catalog.namespace == NAVIGATION_EVENT_NAMESPACE));
        assert!(report
            .package_manifest
            .components
            .iter()
            .any(|component| component.type_id == NAV_MESH_AGENT_COMPONENT_TYPE));
        assert_eq!(
            report.package_manifest.modules[0].target_modes,
            vec![
                zircon_runtime::RuntimeTargetMode::ClientRuntime,
                zircon_runtime::RuntimeTargetMode::ServerRuntime,
                zircon_runtime::RuntimeTargetMode::EditorHost,
            ]
        );
    }

    #[test]
    fn navigation_module_resolves_manager_and_queries_loaded_navmesh() {
        let runtime = CoreRuntime::new();
        runtime.register_module(module_descriptor()).unwrap();
        runtime.activate_module(NAVIGATION_MODULE_NAME).unwrap();
        let manager = runtime
            .handle()
            .resolve_manager::<DefaultNavigationManager>(NAVIGATION_MANAGER_NAME)
            .unwrap();

        let handle = manager
            .load_nav_mesh(NavMeshAsset::simple_quad("humanoid", 5.0))
            .unwrap();
        let mut query = NavPathQuery::new([0.0, 0.0, 0.0], [3.0, 0.0, 4.0]);
        query.nav_mesh = Some(handle);
        let path = manager.find_path(query).unwrap();

        assert_eq!(path.status, NavPathStatus::Complete);
        assert_eq!(path.points.len(), 2);
        assert_eq!(path.length, 5.0);
    }

    #[test]
    fn navigation_manager_ticks_dynamic_agents_toward_destination() {
        let manager = DefaultNavigationManager::new();
        let mut world = World::new();
        world
            .register_component_type(navigation_component_descriptors()[2].clone())
            .unwrap();
        let agent = world.spawn_node(NodeKind::Cube);
        world
            .update_transform(agent, Transform::from_translation(Vec3::ZERO))
            .unwrap();

        let component = serde_json::to_value(NavMeshAgentDescriptor {
            destination: Some([3.0, 0.0, 0.0]),
            speed: 2.0,
            ..NavMeshAgentDescriptor::default()
        })
        .unwrap();
        world
            .set_dynamic_component(agent, NAV_MESH_AGENT_COMPONENT_TYPE, component)
            .unwrap();

        let report = manager.tick_world_agents(&mut world, 0.5).unwrap();

        assert_eq!(report.scanned_agents, 1);
        assert_eq!(report.moved_agents, 1);
        assert_eq!(
            world.world_transform(agent).unwrap().translation,
            Vec3::new(1.0, 0.0, 0.0)
        );
    }

    #[test]
    fn navigation_dynamic_component_descriptor_accepts_vec_and_resource_json() {
        let mut world = World::new();
        let entity = world.spawn_node(NodeKind::Cube);
        world
            .register_component_type(navigation_component_descriptors()[0].clone())
            .unwrap();
        world
            .set_dynamic_component(
                entity,
                NAV_MESH_SURFACE_COMPONENT_TYPE,
                json!({
                    "volume_center": [1.0, 2.0, 3.0],
                    "output_asset": {"resource": "res://navigation/level.znavmesh"}
                }),
            )
            .unwrap();

        assert!(world
            .dynamic_component(entity, NAV_MESH_SURFACE_COMPONENT_TYPE)
            .is_some());
    }

    #[test]
    fn bake_surface_accepts_typed_resource_json_from_dynamic_properties() {
        let manager = DefaultNavigationManager::new();
        let mut world = World::new();
        let entity = world.spawn_node(NodeKind::Cube);
        world
            .register_component_type(navigation_component_descriptors()[0].clone())
            .unwrap();
        world
            .set_dynamic_component(
                entity,
                NAV_MESH_SURFACE_COMPONENT_TYPE,
                json!({
                    "enabled": true,
                    "volume_size": [4.0, 2.0, 4.0],
                    "output_asset": {"resource": "res://navigation/level.znavmesh"}
                }),
            )
            .unwrap();

        let report = manager
            .bake_surface(&world, NavMeshBakeRequest::default())
            .unwrap();

        assert_eq!(
            report.output_asset.as_deref(),
            Some("res://navigation/level.znavmesh")
        );
        assert_eq!(report.surfaces, 1);
        assert_eq!(report.source_triangles, 2);
        assert_eq!(report.baked_polygons, 2);
    }

    #[test]
    fn bake_surface_applies_modifier_area_and_embeds_offmesh_links() {
        let manager = DefaultNavigationManager::new();
        let mut world = World::new();
        for descriptor in navigation_component_descriptors() {
            world.register_component_type(descriptor).unwrap();
        }
        let surface = world.spawn_node(NodeKind::Cube);
        let link = world.spawn_node(NodeKind::Cube);
        world
            .update_transform(link, Transform::from_translation(Vec3::new(4.0, 0.0, 0.0)))
            .unwrap();
        world
            .set_dynamic_component(
                surface,
                NAV_MESH_SURFACE_COMPONENT_TYPE,
                json!({
                    "enabled": true,
                    "volume_size": [4.0, 2.0, 4.0]
                }),
            )
            .unwrap();
        world
            .set_dynamic_component(
                surface,
                NAV_MESH_MODIFIER_COMPONENT_TYPE,
                json!({
                    "override_area": true,
                    "area": AREA_JUMP
                }),
            )
            .unwrap();
        world
            .set_dynamic_component(
                link,
                NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE,
                json!({
                    "activated": true,
                    "agent_type": "humanoid",
                    "start_local_point": [0.0, 0.0, 0.0],
                    "end_local_point": [2.0, 0.0, 0.0],
                    "bidirectional": true,
                    "area_type": AREA_JUMP
                }),
            )
            .unwrap();

        let report = manager
            .bake_surface(&world, NavMeshBakeRequest::default())
            .unwrap();
        let source_triangles = report.source_triangles;
        let asset = report.asset.unwrap();

        assert_eq!(source_triangles, 2);
        assert!(asset
            .polygons
            .iter()
            .all(|polygon| polygon.area == AREA_JUMP));
        assert_eq!(asset.off_mesh_links.len(), 1);
    }
}
