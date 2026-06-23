use zircon_runtime::core::framework::navigation::{
    NAV_MESH_AGENT_COMPONENT_TYPE, NAV_MESH_MODIFIER_COMPONENT_TYPE,
    NAV_MESH_OBSTACLE_COMPONENT_TYPE, NAV_MESH_OFF_MESH_BRIDGE_COMPONENT_TYPE,
    NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE, NAV_MESH_SURFACE_COMPONENT_TYPE,
};

use crate::{
    package_manifest, plugin_registration, NAVIGATION_DIST_CRATE_NAME,
    NAVIGATION_DIST_RUNTIME_ENTRY, NAVIGATION_EVENT_NAMESPACE, NAVIGATION_MODULE_NAME,
    RUNTIME_CAPABILITIES,
};

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
        event.id == "navigation.runtime.navmesh_baked"
            && event.payload_schema == "navigation.runtime.navmesh_bake_report.v1"
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
            zircon_runtime::builtin::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::builtin::RuntimeTargetMode::ServerRuntime,
            zircon_runtime::builtin::RuntimeTargetMode::EditorHost,
        ]
    );
    assert_eq!(report.package_manifest.category, "runtime");
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
fn navigation_package_manifest_declares_dist_contract() {
    let manifest = package_manifest();

    assert!(manifest
        .default_packaging
        .contains(&zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic));

    let distribution = manifest
        .distribution
        .as_ref()
        .expect("navigation distribution manifest");
    assert_eq!(distribution.forms, vec!["dist".to_string()]);
    assert_eq!(
        distribution.default_packaging,
        vec![zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic]
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
            zircon_runtime::builtin::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::builtin::RuntimeTargetMode::ServerRuntime,
            zircon_runtime::builtin::RuntimeTargetMode::EditorHost,
        ]
    );
    for capability in RUNTIME_CAPABILITIES {
        assert!(native_module.capabilities.contains(&capability.to_string()));
    }
}
