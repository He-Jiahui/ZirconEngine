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
