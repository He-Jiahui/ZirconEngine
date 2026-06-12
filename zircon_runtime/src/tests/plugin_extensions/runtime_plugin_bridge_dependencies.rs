use crate::plugin::{
    PluginDependencyManifest, PluginPackageManifest, RuntimePluginCatalog,
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
    assert!(catalog.diagnostics().iter().any(|diagnostic| diagnostic
        .contains("bridge.strong_dependency_missing")
        && diagnostic.contains("package `weather`")
        && diagnostic.contains("provider plugin `physics` is not registered")
        && diagnostic.contains("interface `physics.query.v1`")
        && diagnostic.contains("chain: weather -> physics")));
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
    assert!(catalog.diagnostics().iter().any(|diagnostic| diagnostic
        .contains("bridge.strong_dependency_missing")
        && diagnostic.contains("package `weather`")
        && diagnostic.contains("provider plugin `scene` is not registered")
        && diagnostic.contains("interface `scene.query.v1`")
        && diagnostic.contains("chain: weather -> physics -> scene")));
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

fn registration(manifest: PluginPackageManifest) -> RuntimePluginRegistrationReport {
    RuntimePluginRegistrationReport::from_native_package_manifest(manifest)
}

fn package(id: &str, display_name: &str) -> PluginPackageManifest {
    PluginPackageManifest::new(id, display_name).with_capability(format!("runtime.plugin.{id}"))
}
