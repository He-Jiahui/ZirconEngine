use zircon_runtime::core::framework::navigation::{
    OffMeshTraverseEvent, NAV_DESIRED_VELOCITY_COMPONENT_TYPE, NAV_MESH_AGENT_COMPONENT_TYPE,
    NAV_MESH_MODIFIER_COMPONENT_TYPE, NAV_MESH_OBSTACLE_COMPONENT_TYPE,
    NAV_MESH_OFF_MESH_BRIDGE_COMPONENT_TYPE, NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE,
    NAV_MESH_SURFACE_COMPONENT_TYPE,
};
use zircon_runtime::core::manager::{ManagerResolver, NAVIGATION_MANAGER_NAME};
use zircon_runtime::core::runtime::CoreRuntime;
use zircon_runtime::core::ServiceKind;
use zircon_runtime::scene::ecs::{SystemOrderingConstraint, SystemRef};
use zircon_runtime::scene::{
    SceneNavigationRuntimeHandle, SystemStage, SCENE_NAVIGATION_RUNTIME_DRIVER_NAME,
};

use crate::{
    module_descriptor, package_manifest, plugin_registration, DefaultNavigationManager,
    NavigationOverlayFrame, NAVIGATION_DIST_CRATE_NAME, NAVIGATION_DIST_RUNTIME_ENTRY,
    NAVIGATION_EVENT_NAMESPACE, NAVIGATION_MAIN_SYSTEM_SET, NAVIGATION_MODULE_NAME,
    NAVIGATION_OVERLAY_FRAME_EVENT_ID, NAVIGATION_OVERLAY_FRAME_PAYLOAD_SCHEMA,
    RUNTIME_CAPABILITIES,
};

#[test]
fn navigation_module_obeys_driver_manager_dependency_layers() {
    const IMPLEMENTATION_DRIVER_NAME: &str = "navigation.runtime.Driver.DefaultNavigationRuntime";

    let descriptor = module_descriptor();

    let implementation = descriptor
        .drivers
        .iter()
        .find(|driver| driver.name.as_str() == IMPLEMENTATION_DRIVER_NAME)
        .expect("navigation implementation must be registered as a driver");
    assert!(implementation.dependencies.is_empty());

    let scene_driver = descriptor
        .drivers
        .iter()
        .find(|driver| driver.name.as_str() == SCENE_NAVIGATION_RUNTIME_DRIVER_NAME)
        .expect("scene navigation runtime must be registered as a driver");
    assert_eq!(scene_driver.dependencies.len(), 1);
    assert_eq!(
        scene_driver.dependencies[0].name.as_str(),
        IMPLEMENTATION_DRIVER_NAME
    );
    assert_eq!(
        scene_driver.dependencies[0].name.service_kind(),
        ServiceKind::Driver
    );

    let public_manager = descriptor
        .managers
        .iter()
        .find(|manager| manager.name.as_str() == NAVIGATION_MANAGER_NAME)
        .expect("public navigation facade must be registered as a manager");
    assert_eq!(public_manager.dependencies.len(), 1);
    assert_eq!(
        public_manager.dependencies[0].name.as_str(),
        IMPLEMENTATION_DRIVER_NAME
    );

    let runtime = CoreRuntime::new();
    runtime
        .register_module(descriptor)
        .expect("navigation service dependency layering must be valid");
    runtime
        .resolve_driver::<DefaultNavigationManager>(IMPLEMENTATION_DRIVER_NAME)
        .expect("navigation implementation driver must resolve");
    runtime
        .resolve_driver::<SceneNavigationRuntimeHandle>(SCENE_NAVIGATION_RUNTIME_DRIVER_NAME)
        .expect("scene navigation runtime driver must resolve");
    let resolver = ManagerResolver::new(runtime.handle());
    resolver
        .resolve(
            resolver
                .navigation_handle()
                .expect("public navigation manager handle"),
        )
        .expect("public navigation manager facade must resolve");
}

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
        NAV_MESH_OFF_MESH_BRIDGE_COMPONENT_TYPE,
    ] {
        assert!(report
            .extensions
            .components()
            .iter()
            .any(|component| component.type_id == component_type));
    }
    let default_settings = report
        .extensions
        .plugin_options()
        .iter()
        .find(|option| option.key == "navigation.default_settings_asset")
        .expect("navigation default settings option");
    assert_eq!(default_settings.value_type, "string");
    assert_eq!(
        default_settings.default_value,
        "res://navigation/settings/default.navigation.toml"
    );
    let bake_backend = report
        .extensions
        .plugin_options()
        .iter()
        .find(|option| option.key == "navigation.bake_backend")
        .expect("navigation bake backend option");
    assert_eq!(bake_backend.value_type, "enum");
    assert_eq!(bake_backend.default_value, "recast");
    assert_eq!(bake_backend.enum_values, vec!["recast".to_string()]);
    assert_eq!(
        bake_backend.required_capability.as_deref(),
        Some("runtime.plugin.navigation.recast")
    );
    let event_catalog = report
        .extensions
        .plugin_event_catalogs()
        .iter()
        .find(|catalog| catalog.namespace == NAVIGATION_EVENT_NAMESPACE)
        .expect("navigation runtime event catalog");
    assert!(event_catalog.events.iter().any(|event| {
        event.id == "navigation.events.navmesh_baked"
            && event.payload_schema == "navigation.events.navmesh_bake_report.v1"
    }));
    assert!(event_catalog.events.iter().any(|event| {
        event.id == "navigation.events.off_mesh_traverse"
            && event.payload_schema == "navigation.events.off_mesh_traverse.v1"
    }));
    assert!(event_catalog.events.iter().any(|event| {
        event.id == NAVIGATION_OVERLAY_FRAME_EVENT_ID
            && event.payload_schema == NAVIGATION_OVERLAY_FRAME_PAYLOAD_SCHEMA
    }));
    assert!(report.extensions.plugin_events().any(|(_, event)| {
        event
            .type_name()
            .ends_with(std::any::type_name::<OffMeshTraverseEvent>())
    }));
    assert!(report.extensions.plugin_events().any(|(_, event)| {
        event
            .type_name()
            .ends_with(std::any::type_name::<NavigationOverlayFrame>())
    }));
    assert!(report
        .package_manifest
        .components
        .iter()
        .any(|component| component.type_id == NAV_MESH_AGENT_COMPONENT_TYPE));
    assert!(report
        .package_manifest
        .components
        .iter()
        .any(|component| component.type_id == NAV_MESH_OFF_MESH_BRIDGE_COMPONENT_TYPE));
    assert_eq!(
        report.package_manifest.modules[0].target_modes,
        vec![
            zircon_runtime::core::framework::platform::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::core::framework::platform::RuntimeTargetMode::ServerRuntime,
            zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost,
        ]
    );
    assert_eq!(report.package_manifest.category, "runtime");
    assert_eq!(
        report.package_manifest.modules[0].system_anchors,
        vec!["navigation.agent_tick".to_string()]
    );
    assert_eq!(
        report.package_manifest.maturity,
        zircon_runtime::plugin::PluginMaturity::Beta
    );
    assert!(report
        .package_manifest
        .capabilities
        .contains(&"runtime.plugin.navigation.recast".to_string()));
    assert!(report.package_manifest.modules[0]
        .capabilities
        .contains(&"runtime.plugin.navigation.recast".to_string()));
    assert!(report
        .package_manifest
        .capability_statuses
        .iter()
        .any(|status| {
            status.capability == "runtime.plugin.navigation"
                && status.status == zircon_runtime::plugin::CapabilityStatus::Partial
                && status.note.as_deref()
                    == Some(
                        "Gameplay navmesh/pathfinding is optional; UI navigation parity is separate.",
                    )
        }));
}

#[test]
fn agent_tick_registered_after_ai_behavior_tick() {
    let report = plugin_registration();
    let system = report
        .extensions
        .plugin_runtime_systems()
        .find_map(|(_, system)| (system.id == "navigation.agent_tick").then_some(system))
        .expect("navigation agent tick runtime system");

    assert_eq!(system.stage, SystemStage::Update);
    assert!(system
        .constraints
        .contains(&SystemOrderingConstraint::After(SystemRef::System(
            "ai.behavior_tick".to_string()
        ))));
    assert!(report
        .extensions
        .components()
        .iter()
        .any(|component| { component.type_id == NAV_DESIRED_VELOCITY_COMPONENT_TYPE }));
    assert!(report.extensions.plugin_resources().any(|(_, resource)| {
        resource
            .type_name()
            .ends_with("navigation::repath_budget::NavRepathBudget")
    }));
    assert!(report.extensions.plugin_resources().any(|(_, resource)| {
        resource
            .type_name()
            .ends_with("navigation::agent::NavigationDebugCapture")
    }));
}

#[test]
fn navigation_runtime_systems_join_main_system_set() {
    assert_eq!(NAVIGATION_MAIN_SYSTEM_SET, "navigation.main");

    let mut report = plugin_registration();
    let runtime_module = report
        .package_manifest
        .modules
        .iter()
        .find(|module| module.name == NAVIGATION_MODULE_NAME)
        .expect("navigation.runtime module");
    assert_eq!(
        runtime_module.system_sets,
        vec![NAVIGATION_MAIN_SYSTEM_SET.to_string()]
    );

    let main_set = report
        .extensions
        .intern_system_set(NAVIGATION_MAIN_SYSTEM_SET)
        .expect("navigation.main should be a valid system set");
    let runtime_systems = report
        .extensions
        .plugin_runtime_systems()
        .filter(|(owner, _)| {
            report.extensions.plugin_module_name(*owner) == Some(NAVIGATION_MODULE_NAME)
        })
        .map(|(_, system)| system)
        .collect::<Vec<_>>();
    assert!(!runtime_systems.is_empty());
    for system in runtime_systems {
        assert_eq!(
            system.sets,
            vec![main_set],
            "{} must join navigation.main",
            system.id
        );
    }
}

#[test]
fn navigation_package_manifest_declares_dist_contract() {
    let manifest = package_manifest();

    assert!(manifest.default_packaging.contains(
        &zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic
    ));

    let distribution = manifest
        .distribution
        .as_ref()
        .expect("navigation distribution manifest");
    assert_eq!(distribution.forms, vec!["dist".to_string()]);
    assert_eq!(
        distribution.default_packaging,
        vec![zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic]
    );
    assert_eq!(distribution.abi_version, Some(3));
    assert_eq!(distribution.engine_compat, ">=0.1, <0.2");
    assert_eq!(distribution.dist_crate, NAVIGATION_DIST_CRATE_NAME);
    assert_eq!(
        distribution.descriptor_symbol,
        "zircon_native_plugin_descriptor_v3"
    );
    assert_eq!(distribution.runtime_entry, NAVIGATION_DIST_RUNTIME_ENTRY);

    let native_module = manifest
        .modules
        .iter()
        .find(|module| module.name == "navigation.dist")
        .expect("navigation native dist module");
    assert_eq!(
        native_module.kind,
        zircon_runtime::plugin::PluginModuleKind::Native
    );
    assert_eq!(native_module.crate_name, NAVIGATION_DIST_CRATE_NAME);
    assert_eq!(
        native_module.target_modes,
        vec![
            zircon_runtime::core::framework::platform::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::core::framework::platform::RuntimeTargetMode::ServerRuntime,
            zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost,
        ]
    );
    for capability in RUNTIME_CAPABILITIES {
        assert!(native_module.capabilities.contains(&capability.to_string()));
    }
}
