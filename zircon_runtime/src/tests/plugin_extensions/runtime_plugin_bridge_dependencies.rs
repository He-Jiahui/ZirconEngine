use std::sync::Arc;

use crate::core::framework::bridge::{BridgeError, BridgeOwnerTransitionMode, PluginInterface};
use crate::core::framework::project::{ExportPackagingStrategy, ProjectPluginSelection};
use crate::plugin::{
    PluginDependencyManifest, PluginPackageManifest, RuntimeExtensionRegistry,
    RuntimePluginBridgeLifecycleError, RuntimePluginBridgeLifecycleEvent,
    RuntimePluginBridgeLifecycleOutcome, RuntimePluginBridgeLifecycleState, RuntimePluginCatalog,
    RuntimePluginRegistrationReport,
};

#[test]
fn runtime_plugin_catalog_reports_missing_required_bridge_dependency_interface() {
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [registration(package("weather", "Weather").with_dependency(
            PluginDependencyManifest::new("physics", true).with_interface("physics.query.v1"),
        ))],
        [],
    );

    assert!(!catalog.is_success());
    assert!(catalog.diagnostics().iter().any(|diagnostic| {
        diagnostic.contains("bridge.strong_dependency_missing")
            && diagnostic.contains("package `weather`")
            && diagnostic.contains("provider plugin `physics` is not registered")
            && diagnostic.contains("interface `physics.query.v1`")
            && diagnostic.contains("chain: weather -> physics")
    }));
}

#[test]
fn runtime_plugin_catalog_accepts_registered_required_bridge_dependency_interface() {
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [
            registration(
                package("physics", "Physics").with_provided_interface_id("physics.query.v1"),
            ),
            registration(package("weather", "Weather").with_dependency(
                PluginDependencyManifest::new("physics", true).with_interface("physics.query.v1"),
            )),
        ],
        [],
    );

    assert!(catalog.is_success());
    assert!(catalog.diagnostics().is_empty());
}

#[test]
fn runtime_plugin_catalog_allows_missing_optional_bridge_dependency_interface() {
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [registration(package("weather", "Weather").with_dependency(
            PluginDependencyManifest::new("physics", false).with_interface("physics.query.v1"),
        ))],
        [],
    );

    assert!(catalog.is_success());
    assert!(catalog.diagnostics().is_empty());
}

#[test]
fn runtime_plugin_catalog_reports_transitive_required_bridge_dependency_chain() {
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [
            registration(package("weather", "Weather").with_dependency(
                PluginDependencyManifest::new("physics", true).with_interface("physics.query.v1"),
            )),
            registration(
                package("physics", "Physics")
                    .with_provided_interface_id("physics.query.v1")
                    .with_dependency(
                        PluginDependencyManifest::new("scene", true)
                            .with_interface("scene.query.v1"),
                    ),
            ),
        ],
        [],
    );

    assert!(!catalog.is_success());
    assert!(catalog.diagnostics().iter().any(|diagnostic| {
        diagnostic.contains("bridge.strong_dependency_missing")
            && diagnostic.contains("package `weather`")
            && diagnostic.contains("provider plugin `scene` is not registered")
            && diagnostic.contains("interface `scene.query.v1`")
            && diagnostic.contains("chain: weather -> physics -> scene")
    }));
}

#[test]
fn runtime_plugin_catalog_lists_strong_bridge_dependents_for_disable_checks() {
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [
            registration(
                package("physics", "Physics")
                    .with_provided_interface_id("physics.query.v1")
                    .with_provided_interface_id("physics.force.v1"),
            ),
            registration(
                package("weather", "Weather").with_dependency(
                    PluginDependencyManifest::new("physics", true)
                        .with_interfaces(["physics.force.v1", "physics.query.v1"]),
                ),
            ),
            registration(package("ai", "AI").with_dependency(
                PluginDependencyManifest::new("physics", true).with_interface("physics.query.v1"),
            )),
            registration(package("sound", "Sound").with_dependency(
                PluginDependencyManifest::new("physics", false).with_interface("physics.query.v1"),
            )),
        ],
        [],
    );

    let dependents = catalog.strong_bridge_dependents("physics");

    assert_eq!(dependents.len(), 2);
    assert_eq!(dependents[0].package_id, "ai");
    assert_eq!(dependents[0].interface_ids, vec!["physics.query.v1"]);
    assert_eq!(dependents[1].package_id, "weather");
    assert_eq!(
        dependents[1].interface_ids,
        vec!["physics.force.v1", "physics.query.v1"]
    );
}

#[test]
fn runtime_plugin_catalog_reports_strong_bridge_disable_blockers() {
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [
            registration(
                package("physics", "Physics").with_provided_interface_id("physics.query.v1"),
            ),
            registration(package("weather", "Weather").with_dependency(
                PluginDependencyManifest::new("physics", true).with_interface("physics.query.v1"),
            )),
        ],
        [],
    );

    let blockers = catalog.strong_bridge_disable_blockers("physics");

    assert_eq!(blockers.len(), 1);
    assert_eq!(blockers[0].provider_package_id, "physics");
    assert_eq!(blockers[0].dependent_package_id, "weather");
    assert_eq!(blockers[0].interface_ids, vec!["physics.query.v1"]);
    assert_eq!(
        blockers[0].diagnostic(),
        "bridge.strong_target_disable_blocked: provider plugin `physics` cannot be disabled while dependent plugin `weather` requires interfaces [`physics.query.v1`]"
    );
    assert!(catalog.strong_bridge_disable_blockers("weather").is_empty());
}

#[test]
fn runtime_plugin_catalog_merges_bridge_exports_into_final_registry() {
    let mut extensions = RuntimeExtensionRegistry::default();
    let owner = extensions.intern_plugin_module("physics.runtime").unwrap();
    extensions
        .export_interface::<dyn PhysicsQueryInterface>(owner, Arc::new(PhysicsProvider))
        .unwrap();
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [registration_with_extensions(
            package_with_runtime("physics", "Physics")
                .with_provided_interface_id("physics.query.v1"),
            extensions,
        )],
        [],
    );

    let report = catalog.runtime_extensions();

    assert!(report.fatal_diagnostics.is_empty());
    assert!(report.diagnostics.is_empty());
    let table = report.registry.frozen_bridge_table();
    let bridge = table.resolve_weak::<dyn PhysicsQueryInterface>();
    assert_eq!(bridge.call(|provider| provider.ray_count()), Ok(42));
}

#[test]
fn disable_strong_target_rejected_with_dependents() {
    let mut physics_extensions = RuntimeExtensionRegistry::default();
    let owner = physics_extensions
        .intern_plugin_module("physics.runtime")
        .unwrap();
    physics_extensions
        .export_interface::<dyn PhysicsQueryInterface>(owner, Arc::new(PhysicsProvider))
        .unwrap();
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [
            registration_with_extensions(
                package_with_runtime("physics", "Physics")
                    .with_provided_interface_id("physics.query.v1"),
                physics_extensions,
            ),
            registration(package("weather", "Weather").with_dependency(
                PluginDependencyManifest::new("physics", true).with_interface("physics.query.v1"),
            )),
        ],
        [],
    );
    let report = catalog.runtime_extensions();
    let table = report.registry.frozen_bridge_table();
    let bridge = table.resolve_weak::<dyn PhysicsQueryInterface>();

    let error = catalog
        .disable_bridge_provider_at_frame_boundary(&report.registry, &table, "physics")
        .unwrap_err();

    let RuntimePluginBridgeLifecycleError::StrongDependentsBlocked(block) = error;
    assert_eq!(block.provider_package_id, "physics");
    assert_eq!(block.mode, BridgeOwnerTransitionMode::Disable);
    assert_eq!(block.blockers.len(), 1);
    assert_eq!(block.blockers[0].dependent_package_id, "weather");
    assert!(block
        .diagnostic()
        .contains("bridge.strong_target_disable_blocked"));
    assert_eq!(bridge.call(|provider| provider.ray_count()), Ok(42));
}

#[test]
fn bridge_lifecycle_disable_and_activate_flips_provider_at_frame_boundary() {
    let mut physics_extensions = RuntimeExtensionRegistry::default();
    let owner = physics_extensions
        .intern_plugin_module("physics.runtime")
        .unwrap();
    physics_extensions
        .export_interface::<dyn PhysicsQueryInterface>(owner, Arc::new(PhysicsProvider))
        .unwrap();
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [
            registration_with_extensions(
                package_with_runtime("physics", "Physics")
                    .with_provided_interface_id("physics.query.v1"),
                physics_extensions,
            ),
            registration(package("weather", "Weather").with_dependency(
                PluginDependencyManifest::new("physics", false).with_interface("physics.query.v1"),
            )),
        ],
        [],
    );
    let report = catalog.runtime_extensions();
    let table = report.registry.frozen_bridge_table();
    let bridge = table.resolve_weak::<dyn PhysicsQueryInterface>();

    let disable_report = catalog
        .disable_bridge_provider_at_frame_boundary(&report.registry, &table, "physics")
        .expect("optional dependents do not block provider disable");

    assert_eq!(disable_report.provider_package_id, "physics");
    assert_eq!(disable_report.mode, BridgeOwnerTransitionMode::Disable);
    assert_eq!(disable_report.owner_reports.len(), 1);
    assert_eq!(disable_report.affected_slot_count(), 1);
    assert_eq!(
        bridge.call(|provider| provider.ray_count()),
        Err(BridgeError::NotEnabled)
    );

    let activate_report =
        catalog.activate_bridge_provider_at_frame_boundary(&report.registry, &table, "physics");

    assert_eq!(activate_report.mode, BridgeOwnerTransitionMode::Activate);
    assert_eq!(activate_report.affected_slot_count(), 1);
    assert_eq!(bridge.call(|provider| provider.ray_count()), Ok(42));
}

#[test]
fn bridge_lifecycle_reload_replaces_provider_from_reloaded_registry() {
    let mut physics_extensions = RuntimeExtensionRegistry::default();
    let owner = physics_extensions
        .intern_plugin_module("physics.runtime")
        .unwrap();
    physics_extensions
        .export_interface::<dyn PhysicsQueryInterface>(owner, Arc::new(PhysicsProvider))
        .unwrap();
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [registration_with_extensions(
            package_with_runtime("physics", "Physics")
                .with_provided_interface_id("physics.query.v1"),
            physics_extensions,
        )],
        [],
    );
    let report = catalog.runtime_extensions();
    let table = report.registry.frozen_bridge_table();
    let slot = table
        .resolve_slot(<dyn PhysicsQueryInterface as PluginInterface>::INTERFACE_ID)
        .expect("physics bridge slot");
    let bridge = table.resolve_weak::<dyn PhysicsQueryInterface>();
    let original_generation = table.entry(slot).unwrap().generation();

    let mut replacement_extensions = RuntimeExtensionRegistry::default();
    let replacement_owner = replacement_extensions
        .intern_plugin_module("physics.runtime")
        .unwrap();
    replacement_extensions
        .export_interface::<dyn PhysicsQueryInterface>(
            replacement_owner,
            Arc::new(PhysicsReloadProvider { rays: 84 }),
        )
        .unwrap();
    let replacement_catalog = RuntimePluginCatalog::from_registration_reports(
        [registration_with_extensions(
            package_with_runtime("physics", "Physics")
                .with_provided_interface_id("physics.query.v1"),
            replacement_extensions,
        )],
        [],
    );
    let replacement_report = replacement_catalog.runtime_extensions();

    let reload_report = catalog.reload_bridge_provider_at_frame_boundary(
        &report.registry,
        &replacement_report.registry,
        &table,
        "physics",
    );

    assert_eq!(reload_report.mode, BridgeOwnerTransitionMode::Reload);
    assert_eq!(reload_report.affected_slot_count(), 1);
    assert_eq!(
        table.entry(slot).unwrap().generation(),
        original_generation + 2
    );
    assert_eq!(bridge.call(|provider| provider.ray_count()), Ok(84));
}

#[test]
fn bridge_lifecycle_state_owns_frozen_table_for_provider_events() {
    let mut physics_extensions = RuntimeExtensionRegistry::default();
    let owner = physics_extensions
        .intern_plugin_module("physics.runtime")
        .unwrap();
    physics_extensions
        .export_interface::<dyn PhysicsQueryInterface>(owner, Arc::new(PhysicsProvider))
        .unwrap();
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [
            registration_with_extensions(
                package_with_runtime("physics", "Physics")
                    .with_provided_interface_id("physics.query.v1"),
                physics_extensions,
            ),
            registration(package("weather", "Weather").with_dependency(
                PluginDependencyManifest::new("physics", false).with_interface("physics.query.v1"),
            )),
        ],
        [],
    );
    let state = RuntimePluginBridgeLifecycleState::from_catalog(catalog);
    let bridge = state
        .bridge_table()
        .resolve_weak::<dyn PhysicsQueryInterface>();

    assert!(state.extension_report().fatal_diagnostics.is_empty());
    assert_eq!(state.diagnostics_summary().enabled_interfaces, 1);
    assert_eq!(bridge.call(|provider| provider.ray_count()), Ok(42));

    let disable_outcome = state.apply_provider_lifecycle_event(
        RuntimePluginBridgeLifecycleEvent::disable_provider("physics"),
    );
    let RuntimePluginBridgeLifecycleOutcome::Applied(disable_report) = disable_outcome else {
        panic!("optional dependents should not block provider disable");
    };

    assert_eq!(disable_report.mode, BridgeOwnerTransitionMode::Disable);
    assert_eq!(disable_report.affected_slot_count(), 1);
    assert!(disable_report
        .diagnostic()
        .contains("bridge.provider_lifecycle"));
    assert_eq!(state.diagnostics_summary().disabled_interfaces, 1);
    assert_eq!(
        bridge.call(|provider| provider.ray_count()),
        Err(BridgeError::NotEnabled)
    );

    let activate_outcome = state.apply_provider_lifecycle_event(
        RuntimePluginBridgeLifecycleEvent::activate_provider("physics"),
    );
    let RuntimePluginBridgeLifecycleOutcome::Applied(activate_report) = activate_outcome else {
        panic!("provider activation should not be blocked");
    };

    assert_eq!(activate_report.mode, BridgeOwnerTransitionMode::Activate);
    assert_eq!(activate_report.affected_slot_count(), 1);
    assert_eq!(state.diagnostics_summary().enabled_interfaces, 1);
    assert_eq!(bridge.call(|provider| provider.ray_count()), Ok(42));

    let slot = state
        .bridge_table()
        .resolve_slot(<dyn PhysicsQueryInterface as PluginInterface>::INTERFACE_ID)
        .expect("physics bridge slot");
    let generation_before_reload = state.bridge_table().entry(slot).unwrap().generation();
    let reload_outcome = state.apply_provider_lifecycle_event(
        RuntimePluginBridgeLifecycleEvent::reload_provider("physics"),
    );
    let RuntimePluginBridgeLifecycleOutcome::Applied(reload_report) = reload_outcome else {
        panic!("provider reload should not be blocked");
    };

    assert_eq!(reload_report.mode, BridgeOwnerTransitionMode::Reload);
    assert_eq!(reload_report.affected_slot_count(), 1);
    assert_eq!(
        state.bridge_table().entry(slot).unwrap().generation(),
        generation_before_reload + 2
    );
    assert_eq!(bridge.call(|provider| provider.ray_count()), Ok(42));
}

#[test]
fn bridge_lifecycle_state_rejects_strong_provider_disable() {
    let mut physics_extensions = RuntimeExtensionRegistry::default();
    let owner = physics_extensions
        .intern_plugin_module("physics.runtime")
        .unwrap();
    physics_extensions
        .export_interface::<dyn PhysicsQueryInterface>(owner, Arc::new(PhysicsProvider))
        .unwrap();
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [
            registration_with_extensions(
                package_with_runtime("physics", "Physics")
                    .with_provided_interface_id("physics.query.v1"),
                physics_extensions,
            ),
            registration(package("weather", "Weather").with_dependency(
                PluginDependencyManifest::new("physics", true).with_interface("physics.query.v1"),
            )),
        ],
        [],
    );
    let state = RuntimePluginBridgeLifecycleState::from_catalog(catalog);
    let bridge = state
        .bridge_table()
        .resolve_weak::<dyn PhysicsQueryInterface>();

    let outcome = state.apply_provider_lifecycle_event(
        RuntimePluginBridgeLifecycleEvent::disable_provider("physics"),
    );

    assert!(!outcome.is_applied());
    assert!(outcome
        .diagnostic()
        .contains("bridge.provider_lifecycle_blocked"));
    let RuntimePluginBridgeLifecycleOutcome::Blocked(error) = outcome else {
        panic!("strong dependents should block provider disable");
    };
    let RuntimePluginBridgeLifecycleError::StrongDependentsBlocked(block) = error;
    assert_eq!(block.provider_package_id, "physics");
    assert_eq!(block.mode, BridgeOwnerTransitionMode::Disable);
    assert_eq!(block.blockers[0].dependent_package_id, "weather");
    assert_eq!(bridge.call(|provider| provider.ray_count()), Ok(42));
}

fn registration(manifest: PluginPackageManifest) -> RuntimePluginRegistrationReport {
    RuntimePluginRegistrationReport::from_native_package_manifest(manifest)
}

fn registration_with_extensions(
    manifest: PluginPackageManifest,
    extensions: RuntimeExtensionRegistry,
) -> RuntimePluginRegistrationReport {
    let project_selection = ProjectPluginSelection {
        id: manifest.id.clone(),
        enabled: true,
        required: false,
        target_modes: Vec::new(),
        packaging: ExportPackagingStrategy::SourceTemplate,
        runtime_crate: None,
        editor_crate: None,
        features: Vec::new(),
    };
    RuntimePluginRegistrationReport {
        package_manifest: manifest,
        project_selection,
        extensions,
        diagnostics: Vec::new(),
    }
}

fn package(id: &str, display_name: &str) -> PluginPackageManifest {
    PluginPackageManifest::new(id, display_name).with_capability(format!("runtime.plugin.{id}"))
}

fn package_with_runtime(id: &str, display_name: &str) -> PluginPackageManifest {
    package(id, display_name).with_runtime_crate(format!("{id}_runtime"))
}

trait PhysicsQueryInterface: Send + Sync {
    fn ray_count(&self) -> u32;
}

impl PluginInterface for dyn PhysicsQueryInterface {
    const INTERFACE_ID: &'static str = "physics.query.v1";
}

#[derive(Debug)]
struct PhysicsProvider;

impl PhysicsQueryInterface for PhysicsProvider {
    fn ray_count(&self) -> u32 {
        42
    }
}

#[derive(Debug)]
struct PhysicsReloadProvider {
    rays: u32,
}

impl PhysicsQueryInterface for PhysicsReloadProvider {
    fn ray_count(&self) -> u32 {
        self.rays
    }
}
