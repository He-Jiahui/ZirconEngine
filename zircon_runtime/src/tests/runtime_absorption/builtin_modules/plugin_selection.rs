use crate::{
    builtin::{runtime_modules_for_target, RuntimePluginId},
    core::framework::platform::RuntimeTargetMode,
    core::framework::project::ProjectPluginManifest,
    core::framework::project::ProjectPluginSelection,
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
        .any(|missing| missing.runtime_id == RuntimePluginId::VirtualGeometry));
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
    assert!(report.warning_messages().iter().any(|warning| warning
        .contains("optional runtime plugin VirtualGeometry is unavailable")
        && warning.contains("no linked or native dynamic provider registration")));
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
        .any(|missing| missing.runtime_id == RuntimePluginId::Physics));
    assert!(report
        .required_missing()
        .iter()
        .any(|missing| missing.runtime_id == RuntimePluginId::Animation));
    let fatal_messages = report.fatal_messages();
    assert!(fatal_messages.iter().any(|error| error.contains("Physics")));
    assert!(report
        .fatal_messages()
        .iter()
        .any(|error| error.contains("Animation")));
}
