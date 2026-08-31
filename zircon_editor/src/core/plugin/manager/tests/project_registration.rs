//! Project-native registration replacement and host-owned lifecycle regressions.

use zircon_runtime::plugin::{PluginModuleManifest, PluginPackageManifest};

use crate::core::editor_extension::EditorExtensionRegistry;
use crate::core::plugin::sdk::lifecycle::{
    EditorPluginLifecycleReport, EditorPluginLifecycleStage,
};
use crate::core::plugin::{EditorPluginDescriptor, EditorPluginRegistrationReport};
use crate::core::runtime_event_consumer::EditorRuntimeEventConsumerRegistry;

use super::super::{
    EditorPluginCatalog, EditorPluginLoadingPhase, EditorPluginManager, EditorPluginSource,
    EditorPluginState,
};

#[test]
fn project_native_reports_replace_only_project_rows_and_record_host_lifecycle() {
    let manager = EditorPluginManager::new(EditorPluginCatalog::from_descriptors(
        [EditorPluginDescriptor::new(
            "fixture.builtin",
            "Fixture builtin",
            "fixture_builtin",
        )],
        [],
    ))
    .expect("the builtin fixture manager should be admissible");
    manager
        .advance_loading_phase(EditorPluginLoadingPhase::Default)
        .expect("the default phase should be reachable for project reports");

    manager
        .publish_project_registration_reports([host_owned_project_report("fixture.project.one")])
        .expect("the first project-native report should publish");
    let first = manager.state_snapshot();
    let first_entry = first
        .entry("fixture.project.one")
        .expect("the project report should be present in the manager");
    assert_eq!(first_entry.source(), EditorPluginSource::Project);
    assert_eq!(first_entry.state(), EditorPluginState::Active);
    assert_eq!(
        first.entry("fixture.builtin").map(|entry| entry.source()),
        Some(EditorPluginSource::Builtin)
    );
    let first_registration = first
        .catalog_snapshot()
        .registration("fixture.project.one")
        .expect("the project registration should remain inspectable");
    assert!(first_registration.lifecycle_stage_succeeded(&EditorPluginLifecycleStage::Loaded));
    assert!(first_registration.lifecycle_stage_succeeded(&EditorPluginLifecycleStage::Enabled));

    manager
        .publish_project_registration_reports([host_owned_project_report("fixture.project.two")])
        .expect("reopening another project should retract the prior project source");
    let replacement = manager.state_snapshot();
    assert!(replacement.entry("fixture.project.one").is_none());
    assert!(replacement.entry("fixture.builtin").is_some());
    assert_eq!(
        replacement
            .entry("fixture.project.two")
            .map(|entry| entry.source()),
        Some(EditorPluginSource::Project)
    );

    manager
        .publish_project_registration_reports([host_owned_project_report("fixture.project.two")])
        .expect("republishing the current project should reinitialize its host lifecycle");
    let refreshed = manager.state_snapshot();
    let refreshed_registration = refreshed
        .catalog_snapshot()
        .registration("fixture.project.two")
        .expect("the refreshed project registration should remain inspectable");
    assert!(refreshed_registration.lifecycle_stage_succeeded(&EditorPluginLifecycleStage::Loaded));
    assert!(refreshed_registration.lifecycle_stage_succeeded(&EditorPluginLifecycleStage::Enabled));

    let receipt = manager
        .clear_project_registration_reports()
        .expect("closing the project should clear its native registrations");
    let cleared = manager.state_snapshot();
    assert!(receipt.is_terminal());
    assert_eq!(
        receipt.retired_package_ids(),
        &["fixture.project.two".to_string()]
    );
    assert!(receipt.remaining_project_package_ids().is_empty());
    assert_eq!(
        receipt.previous_manager_generation(),
        refreshed.generation()
    );
    assert_eq!(
        receipt.previous_catalog_generation(),
        refreshed.catalog_generation()
    );
    assert_eq!(receipt.manager_generation(), cleared.generation());
    assert_eq!(receipt.catalog_generation(), cleared.catalog_generation());
    assert!(cleared.entry("fixture.project.two").is_none());
    assert!(cleared.entry("fixture.builtin").is_some());
}

#[test]
fn clearing_an_empty_project_registration_set_returns_one_terminal_generation() {
    let manager = EditorPluginManager::new(EditorPluginCatalog::from_descriptors([], []))
        .expect("empty manager should be admissible");

    let receipt = manager
        .clear_project_registration_reports()
        .expect("an already empty project registration set is terminal");
    let terminal = manager.state_snapshot();

    assert!(receipt.is_terminal());
    assert!(receipt.retired_package_ids().is_empty());
    assert!(receipt.remaining_project_package_ids().is_empty());
    assert_eq!(receipt.previous_manager_generation(), terminal.generation());
    assert_eq!(
        receipt.previous_catalog_generation(),
        terminal.catalog_generation()
    );
    assert_eq!(receipt.manager_generation(), terminal.generation());
    assert_eq!(receipt.catalog_generation(), terminal.catalog_generation());
}

fn host_owned_project_report(package_id: &str) -> EditorPluginRegistrationReport {
    EditorPluginRegistrationReport {
        package_manifest: PluginPackageManifest::new(package_id, package_id).with_module(
            PluginModuleManifest::editor(
                format!("{package_id}.editor"),
                package_id.replace('.', "_"),
            ),
        ),
        capabilities: Vec::new(),
        extensions: EditorExtensionRegistry::default(),
        lifecycle: EditorPluginLifecycleReport::default(),
        successful_lifecycle_stages: Vec::new(),
        failed_lifecycle_stages: Vec::new(),
        runtime_event_consumers: EditorRuntimeEventConsumerRegistry::default(),
        native_command_bindings: Default::default(),
        diagnostics: Vec::new(),
    }
}
