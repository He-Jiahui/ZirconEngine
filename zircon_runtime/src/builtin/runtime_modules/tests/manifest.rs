use crate::builtin::{default_manifest_for_target, manifest_with_mode_baseline, RuntimePluginId};
use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::{ProjectPluginManifest, ProjectPluginSelection};

#[test]
fn default_server_manifest_avoids_ui() {
    let manifest = default_manifest_for_target(RuntimeTargetMode::ServerRuntime);
    assert!(manifest
        .selections
        .iter()
        .all(|selection| selection.id != RuntimePluginId::Ui.key()));
    assert!(manifest
        .selections
        .iter()
        .all(|selection| selection.id != RuntimePluginId::UiDocumentImporter.key()));
}

#[cfg(feature = "ui")]
#[test]
fn default_ui_manifests_require_the_document_importer_provider() {
    for target in [
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ] {
        let manifest = default_manifest_for_target(target);
        let importer = manifest
            .selections
            .iter()
            .find(|selection| selection.id == RuntimePluginId::UiDocumentImporter.key())
            .expect("UI-capable targets should select the document importer provider");

        assert!(importer.enabled);
        assert!(importer.required);
    }
}

#[test]
fn project_manifest_overlays_mode_baseline() {
    let manifest = manifest_with_mode_baseline(
        RuntimeTargetMode::ClientRuntime,
        Some(&ProjectPluginManifest {
            selections: vec![ProjectPluginSelection::runtime_plugin(
                RuntimePluginId::Physics,
                true,
                false,
            )],
        }),
    );

    #[cfg(feature = "ui")]
    assert!(manifest
        .selections
        .iter()
        .any(|selection| selection.id == RuntimePluginId::Ui.key()));
    #[cfg(not(feature = "ui"))]
    assert!(manifest
        .selections
        .iter()
        .all(|selection| selection.id != RuntimePluginId::Ui.key()));
    assert!(manifest
        .selections
        .iter()
        .any(|selection| selection.id == RuntimePluginId::Physics.key()));
}

#[test]
fn project_manifest_can_disable_mode_baseline_plugin() {
    let manifest = manifest_with_mode_baseline(
        RuntimeTargetMode::ClientRuntime,
        Some(&ProjectPluginManifest {
            selections: vec![ProjectPluginSelection::runtime_plugin(
                RuntimePluginId::Ui,
                false,
                false,
            )],
        }),
    );

    assert!(manifest
        .enabled_for_target(RuntimeTargetMode::ClientRuntime)
        .all(|selection| selection.id != RuntimePluginId::Ui.key()));
}

#[cfg(feature = "ui")]
#[test]
fn project_manifest_can_disable_document_importer_baseline_plugin() {
    let manifest = manifest_with_mode_baseline(
        RuntimeTargetMode::ClientRuntime,
        Some(&ProjectPluginManifest {
            selections: vec![ProjectPluginSelection::runtime_plugin(
                RuntimePluginId::UiDocumentImporter,
                false,
                false,
            )],
        }),
    );

    assert!(manifest
        .enabled_for_target(RuntimeTargetMode::ClientRuntime)
        .all(|selection| selection.id != RuntimePluginId::UiDocumentImporter.key()));
}

#[cfg(feature = "ui")]
#[test]
fn project_manifest_overlay_canonicalizes_runtime_plugin_identity() {
    let mut ui_override = ProjectPluginSelection::runtime_plugin(RuntimePluginId::Ui, true, false);
    ui_override.id = "UI".to_string();

    let manifest = manifest_with_mode_baseline(
        RuntimeTargetMode::ClientRuntime,
        Some(&ProjectPluginManifest {
            selections: vec![ui_override],
        }),
    );
    let ui_selections = manifest
        .selections
        .iter()
        .filter(|selection| RuntimePluginId::parse_key(&selection.id) == Some(RuntimePluginId::Ui))
        .collect::<Vec<_>>();

    assert_eq!(ui_selections.len(), 1);
    assert_eq!(ui_selections[0].id, RuntimePluginId::Ui.key());
    assert!(!ui_selections[0].required);
}

#[test]
fn project_manifest_overlay_preserves_non_baseline_selection_identity() {
    let manifest = manifest_with_mode_baseline(
        RuntimeTargetMode::ClientRuntime,
        Some(&ProjectPluginManifest {
            selections: vec![ProjectPluginSelection::runtime_plugin("audio", true, false)],
        }),
    );

    assert!(manifest
        .selections
        .iter()
        .any(|selection| selection.id == "audio"));
    assert!(manifest
        .selections
        .iter()
        .all(|selection| selection.id != RuntimePluginId::Sound.key()));
}
