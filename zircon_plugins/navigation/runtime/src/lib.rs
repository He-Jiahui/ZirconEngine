use std::sync::Arc;

use zircon_runtime::core::{
    ManagerDescriptor, ModuleDescriptor, ServiceKind, ServiceObject, StartupMode,
};
use zircon_runtime::engine_module::{factory, qualified_name};

mod component_json;
mod components;
mod manager;
mod runtime_obstacles;
mod settings_hash;
mod settings_validation;

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
    use zircon_runtime::asset::{NavMeshAsset, NavMeshLinkAsset, NavigationSettingsAsset};
    use zircon_runtime::core::framework::navigation::{
        NavMeshAgentDescriptor, NavMeshBakeRequest, NavPathQuery, NavPathStatus,
        NavigationAreaSettings, NavigationManager, AREA_JUMP, AREA_WALKABLE, DEFAULT_AGENT_TYPE,
        NAV_MESH_AGENT_COMPONENT_TYPE, NAV_MESH_MODIFIER_COMPONENT_TYPE,
        NAV_MESH_OBSTACLE_COMPONENT_TYPE, NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE,
        NAV_MESH_SURFACE_COMPONENT_TYPE,
    };
    use zircon_runtime::core::math::{Real, Transform, Vec3};
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
    fn navigation_manager_applies_basic_obstacle_avoidance_and_stats() {
        let manager = DefaultNavigationManager::new();
        let mut world = World::new();
        for descriptor in navigation_component_descriptors() {
            world.register_component_type(descriptor).unwrap();
        }
        let agent = world.spawn_node(NodeKind::Cube);
        let obstacle = world.spawn_node(NodeKind::Cube);
        world
            .update_transform(agent, Transform::from_translation(Vec3::ZERO))
            .unwrap();
        world
            .set_dynamic_component(
                agent,
                NAV_MESH_AGENT_COMPONENT_TYPE,
                serde_json::to_value(NavMeshAgentDescriptor {
                    destination: Some([4.0, 0.0, 0.0]),
                    speed: 2.0,
                    ..NavMeshAgentDescriptor::default()
                })
                .unwrap(),
            )
            .unwrap();
        world
            .set_dynamic_component(
                obstacle,
                NAV_MESH_OBSTACLE_COMPONENT_TYPE,
                json!({
                    "shape": "capsule",
                    "center": [1.0, 0.0, 0.2],
                    "radius": 1.0,
                    "avoidance_enabled": true
                }),
            )
            .unwrap();

        let report = manager.tick_world_agents(&mut world, 0.5).unwrap();
        let transform = world.world_transform(agent).unwrap();
        let stats = manager.stats();

        assert_eq!(report.moved_agents, 1);
        assert!(transform.translation.x > 0.0);
        assert!(transform.translation.z < 0.0);
        assert_eq!(stats.active_agents, 1);
        assert_eq!(stats.active_obstacles, 1);
    }

    #[test]
    fn loaded_navmesh_no_path_blocks_agent_instead_of_direct_fallback() {
        let manager = DefaultNavigationManager::new();
        let mut world = World::new();
        world
            .register_component_type(navigation_component_descriptors()[2].clone())
            .unwrap();
        let agent = world.spawn_node(NodeKind::Cube);
        world
            .update_transform(agent, Transform::from_translation(Vec3::ZERO))
            .unwrap();
        world
            .set_dynamic_component(
                agent,
                NAV_MESH_AGENT_COMPONENT_TYPE,
                serde_json::to_value(NavMeshAgentDescriptor {
                    destination: Some([8.0, 0.0, 0.0]),
                    speed: 2.0,
                    ..NavMeshAgentDescriptor::default()
                })
                .unwrap(),
            )
            .unwrap();
        manager.load_nav_mesh(two_island_navmesh(false)).unwrap();

        let report = manager.tick_world_agents(&mut world, 0.5).unwrap();

        assert_eq!(report.moved_agents, 0);
        assert_eq!(report.blocked_agents, 1);
        assert_eq!(
            world.world_transform(agent).unwrap().translation,
            Vec3::ZERO
        );
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("no path")));
    }

    #[test]
    fn carved_runtime_obstacle_blocks_agent_path_on_loaded_navmesh() {
        let manager = DefaultNavigationManager::new();
        let mut world = World::new();
        for descriptor in navigation_component_descriptors() {
            world.register_component_type(descriptor).unwrap();
        }
        let agent = world.spawn_node(NodeKind::Cube);
        let obstacle = world.spawn_node(NodeKind::Cube);
        world
            .update_transform(
                agent,
                Transform::from_translation(Vec3::new(-2.5, 0.0, 0.0)),
            )
            .unwrap();
        world
            .set_dynamic_component(
                agent,
                NAV_MESH_AGENT_COMPONENT_TYPE,
                serde_json::to_value(NavMeshAgentDescriptor {
                    destination: Some([2.5, 0.0, 0.0]),
                    speed: 2.0,
                    ..NavMeshAgentDescriptor::default()
                })
                .unwrap(),
            )
            .unwrap();
        world
            .set_dynamic_component(
                obstacle,
                NAV_MESH_OBSTACLE_COMPONENT_TYPE,
                json!({
                    "shape": "box",
                    "center": [0.0, 0.0, 0.0],
                    // Span the full simple-quad depth so this test proves a block, not a detour.
                    "size": [1.5, 2.0, 7.0],
                    "carve": true,
                    "avoidance_enabled": false
                }),
            )
            .unwrap();
        manager
            .load_nav_mesh(NavMeshAsset::simple_quad("humanoid", 3.0))
            .unwrap();

        let report = manager.tick_world_agents(&mut world, 0.5).unwrap();

        assert_eq!(report.moved_agents, 0);
        assert_eq!(report.blocked_agents, 1);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("no path")));
    }

    #[test]
    fn default_navmesh_selection_uses_first_loaded_handle() {
        let manager = DefaultNavigationManager::new();
        let first = manager.load_nav_mesh(two_island_navmesh(false)).unwrap();
        let second = manager.load_nav_mesh(two_island_navmesh(true)).unwrap();
        assert!(first.0 < second.0);

        let result = manager
            .find_path(NavPathQuery::new([0.0, 0.0, 0.0], [8.0, 0.0, 0.0]))
            .unwrap();

        assert_eq!(result.status, NavPathStatus::NoPath);
    }

    #[test]
    fn invalid_navigation_settings_are_rejected() {
        let manager = DefaultNavigationManager::new();
        let mut duplicate_area = NavigationSettingsAsset::default();
        duplicate_area.areas.push(NavigationAreaSettings {
            id: AREA_WALKABLE,
            name: "duplicate".to_string(),
            cost: 1.0,
            walkable: true,
        });

        let error = manager
            .load_navigation_settings(duplicate_area)
            .unwrap_err();

        assert_eq!(
            error.kind,
            zircon_runtime::core::framework::navigation::NavigationErrorKind::InvalidConfiguration
        );

        let mut bad_cost = NavigationSettingsAsset::default();
        bad_cost.areas[0].cost = Real::NAN;
        assert!(manager.load_navigation_settings(bad_cost).is_err());
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
        assert!(report.baked_polygons > 0);
        assert_eq!(report.tiles, 1);
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

    #[test]
    fn bake_surface_respects_disabled_link_generation_and_settings_hash() {
        let manager = DefaultNavigationManager::new();
        let mut world = World::new();
        for descriptor in navigation_component_descriptors() {
            world.register_component_type(descriptor).unwrap();
        }
        let surface = world.spawn_node(NodeKind::Cube);
        let link = world.spawn_node(NodeKind::Cube);
        world
            .set_dynamic_component(
                surface,
                NAV_MESH_SURFACE_COMPONENT_TYPE,
                json!({
                    "enabled": true,
                    "generate_links": false,
                    "override_voxel_size": 0.25
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
                    "end_local_point": [2.0, 0.0, 0.0]
                }),
            )
            .unwrap();

        let first = manager
            .bake_surface(&world, NavMeshBakeRequest::default())
            .unwrap();
        let first_asset = first.asset.unwrap();
        let mut settings = manager.active_settings();
        settings.areas.push(NavigationAreaSettings {
            id: 7,
            name: "mud".to_string(),
            cost: 5.0,
            walkable: true,
        });
        manager.load_navigation_settings(settings).unwrap();
        let second_asset = manager
            .bake_surface(&world, NavMeshBakeRequest::default())
            .unwrap()
            .asset
            .unwrap();

        assert!(first_asset.off_mesh_links.is_empty());
        assert_ne!(first_asset.settings_hash, 0);
        assert_ne!(first_asset.settings_hash, second_asset.settings_hash);
        assert!(first
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("advanced Recast bake knobs")));
    }

    #[test]
    fn carved_obstacle_removes_static_bake_source() {
        let manager = DefaultNavigationManager::new();
        let mut world = World::new();
        for descriptor in navigation_component_descriptors() {
            world.register_component_type(descriptor).unwrap();
        }
        let surface = world.spawn_node(NodeKind::Cube);
        let obstacle = world.spawn_node(NodeKind::Cube);
        world
            .set_dynamic_component(surface, NAV_MESH_SURFACE_COMPONENT_TYPE, json!({}))
            .unwrap();
        world
            .set_dynamic_component(
                obstacle,
                NAV_MESH_OBSTACLE_COMPONENT_TYPE,
                json!({
                    "shape": "box",
                    "size": [2.0, 2.0, 2.0],
                    "carve": true
                }),
            )
            .unwrap();

        let report = manager
            .bake_surface(&world, NavMeshBakeRequest::default())
            .unwrap();
        let asset = report.asset.unwrap();

        assert_eq!(report.source_triangles, 0);
        assert!(asset.is_empty());
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("carved")));
    }

    fn two_island_navmesh(with_link: bool) -> NavMeshAsset {
        let mut asset = NavMeshAsset::from_triangle_mesh(
            DEFAULT_AGENT_TYPE,
            vec![
                [-1.0, 0.0, -1.0],
                [1.0, 0.0, -1.0],
                [1.0, 0.0, 1.0],
                [-1.0, 0.0, 1.0],
                [7.0, 0.0, -1.0],
                [9.0, 0.0, -1.0],
                [9.0, 0.0, 1.0],
                [7.0, 0.0, 1.0],
            ],
            vec![0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7],
            AREA_WALKABLE,
        );
        if with_link {
            asset.off_mesh_links.push(NavMeshLinkAsset {
                start: [1.0, 0.0, 0.0],
                end: [7.0, 0.0, 0.0],
                width: 0.5,
                bidirectional: true,
                area: AREA_JUMP,
                cost_override: None,
                traversal_mode: Default::default(),
            });
        }
        asset
    }
}
