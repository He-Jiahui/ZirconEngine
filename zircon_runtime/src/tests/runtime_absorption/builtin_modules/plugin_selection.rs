use crate::{
    builtin::{runtime_modules_for_target, RuntimePluginId, RuntimeTargetMode},
    plugin::ProjectPluginManifest,
    plugin::ProjectPluginSelection,
};

#[test]
fn required_unavailable_runtime_plugin_is_reported_as_fatal_missing() {
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::VirtualGeometry,
            true,
            true,
        )],
    };

    let report = runtime_modules_for_target(RuntimeTargetMode::ClientRuntime, Some(&manifest));

    assert!(report
        .required_missing()
        .iter()
        .any(|missing| missing.id == RuntimePluginId::VirtualGeometry));
    assert!(report
        .required_missing_summary()
        .contains("VirtualGeometry"));
}

#[test]
fn optional_unavailable_runtime_plugin_stays_warning_only() {
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::VirtualGeometry,
            true,
            false,
        )],
    };

    let report = runtime_modules_for_target(RuntimeTargetMode::ClientRuntime, Some(&manifest));

    assert!(report.required_missing().is_empty());
    assert!(report
        .warnings
        .iter()
        .any(|warning| warning.contains("zircon_plugins/virtual_geometry")));
}

#[test]
fn physics_animation_manifest_entries_require_linked_external_plugins() {
    let manifest = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Physics, true, true),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Animation, true, true),
        ],
    };

    let report = runtime_modules_for_target(RuntimeTargetMode::ClientRuntime, Some(&manifest));

    assert_eq!(report.required_missing().len(), 2);
    assert!(report
        .required_missing()
        .iter()
        .any(|missing| missing.id == RuntimePluginId::Physics));
    assert!(report
        .required_missing()
        .iter()
        .any(|missing| missing.id == RuntimePluginId::Animation));
    assert!(report.errors.iter().any(|error| error.contains("Physics")));
    assert!(report
        .errors
        .iter()
        .any(|error| error.contains("Animation")));
}
