use zircon_runtime::builtin::RuntimePluginId;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{ProjectPluginManifest, ProjectPluginSelection};

use super::first_party_editor_plugin_registrations_for_manifest;

#[test]
fn editor_catalog_preallocates_manifest_projection_storage() {
    let source = include_str!("catalog.rs");
    let projection = source
        .split("pub fn first_party_editor_plugin_registrations_for_manifest")
        .nth(1)
        .and_then(|text| {
            text.split("pub fn first_party_registration_for_editor_plugin")
                .next()
        })
        .expect("read editor catalog manifest projection");

    assert!(
        projection.contains("HashSet::with_capacity(manifest.selections.len())")
            && projection.contains("Vec::with_capacity(manifest.selections.len())")
            && projection.contains("for selection in manifest.enabled_for_target(target_mode)")
            && projection.contains("registrations.push(registration);")
            && projection.contains("first_party_editor_registration_provider(&plugin_id)")
            && projection.contains("seen.insert(plugin_id)")
            && projection.contains("registrations.push(provider());")
            && !projection.contains("plugin_id.clone()")
            && !projection.contains(".collect()"),
        "editor catalog projection must preallocate storage and move each parsed id into dedup without cloning"
    );
}

#[cfg(feature = "navigation-editor-plugin")]
#[test]
fn selected_navigation_provider_projects_typed_runtime_consumer() {
    let manifest = manifest_with_navigation(true, RuntimeTargetMode::EditorHost);

    let registrations = first_party_editor_plugin_registrations_for_manifest(
        RuntimeTargetMode::EditorHost,
        &manifest,
    );

    assert_eq!(registrations.len(), 1);
    let registration = &registrations[0];
    assert_eq!(registration.package_manifest.id, "navigation");
    assert!(registration.is_success());
    assert_eq!(registration.runtime_event_consumers.manifests().len(), 1);
}

#[cfg(feature = "navigation-editor-plugin")]
#[test]
fn disabled_or_wrong_target_navigation_provider_is_not_projected() {
    let disabled = manifest_with_navigation(false, RuntimeTargetMode::EditorHost);
    assert!(first_party_editor_plugin_registrations_for_manifest(
        RuntimeTargetMode::EditorHost,
        &disabled,
    )
    .is_empty());

    let runtime_only = manifest_with_navigation(true, RuntimeTargetMode::ClientRuntime);
    assert!(first_party_editor_plugin_registrations_for_manifest(
        RuntimeTargetMode::EditorHost,
        &runtime_only,
    )
    .is_empty());
}

#[cfg(feature = "navigation-editor-plugin")]
#[test]
fn duplicate_project_selections_produce_one_editor_registration() {
    let selection =
        ProjectPluginSelection::runtime_plugin(RuntimePluginId::Navigation, true, false)
            .with_target_modes([RuntimeTargetMode::EditorHost]);
    let manifest = ProjectPluginManifest {
        selections: vec![selection.clone(), selection],
    };

    let registrations = first_party_editor_plugin_registrations_for_manifest(
        RuntimeTargetMode::EditorHost,
        &manifest,
    );

    assert_eq!(registrations.len(), 1);
}

#[cfg(feature = "neural-editor-plugin")]
#[test]
fn selected_neural_provider_projects_real_onnx_authoring_contributions() {
    let manifest = manifest_with_neural(true, RuntimeTargetMode::EditorHost);

    let registrations = first_party_editor_plugin_registrations_for_manifest(
        RuntimeTargetMode::EditorHost,
        &manifest,
    );

    assert_eq!(registrations.len(), 1);
    let registration = &registrations[0];
    assert_eq!(registration.package_manifest.id, "neural");
    assert!(registration.is_success());
    assert!(registration
        .capabilities
        .iter()
        .any(|capability| capability == "editor.extension.neural_authoring"));
    assert!(registration.package_manifest.modules.iter().any(|module| {
        module.name == "neural.editor" && module.crate_name == "zircon_plugin_neural_editor"
    }));
}

#[cfg(feature = "neural-editor-plugin")]
#[test]
fn disabled_wrong_target_or_duplicate_neural_selections_project_correctly() {
    let disabled = manifest_with_neural(false, RuntimeTargetMode::EditorHost);
    assert!(first_party_editor_plugin_registrations_for_manifest(
        RuntimeTargetMode::EditorHost,
        &disabled,
    )
    .is_empty());

    let runtime_only = manifest_with_neural(true, RuntimeTargetMode::ClientRuntime);
    assert!(first_party_editor_plugin_registrations_for_manifest(
        RuntimeTargetMode::EditorHost,
        &runtime_only,
    )
    .is_empty());

    let selection =
        ProjectPluginSelection::runtime_plugin(RuntimePluginId::new("neural"), true, false)
            .with_target_modes([RuntimeTargetMode::EditorHost]);
    let duplicate = ProjectPluginManifest {
        selections: vec![selection.clone(), selection],
    };
    assert_eq!(
        first_party_editor_plugin_registrations_for_manifest(
            RuntimeTargetMode::EditorHost,
            &duplicate,
        )
        .len(),
        1
    );
}

#[cfg(feature = "navigation-editor-plugin")]
fn manifest_with_navigation(enabled: bool, target: RuntimeTargetMode) -> ProjectPluginManifest {
    ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Navigation,
            enabled,
            false,
        )
        .with_target_modes([target])],
    }
}

#[cfg(feature = "neural-editor-plugin")]
fn manifest_with_neural(enabled: bool, target: RuntimeTargetMode) -> ProjectPluginManifest {
    ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::new("neural"),
            enabled,
            false,
        )
        .with_target_modes([target])],
    }
}
