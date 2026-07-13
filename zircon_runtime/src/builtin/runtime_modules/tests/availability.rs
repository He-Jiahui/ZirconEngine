use crate::builtin::{runtime_modules_for_runtime_profile, RuntimePluginId};
use crate::core::framework::project::RuntimeProfileId;
#[cfg(not(feature = "ui"))]
use crate::core::framework::project::{ProjectPluginManifest, ProjectPluginSelection};
#[cfg(not(feature = "ui"))]
use crate::{builtin::runtime_modules_for_target, core::framework::platform::RuntimeTargetMode};

use super::support::availability_contains;

#[test]
fn runtime_profile_load_report_surfaces_structured_availability() {
    let report = runtime_modules_for_runtime_profile(RuntimeProfileId::Client2d);

    assert!(availability_contains(
        &report.runtime_plugin_availability.externalized_missing,
        RuntimePluginId::Sound
    ));
    assert!(availability_contains(
        &report.runtime_plugin_availability.missing_required,
        RuntimePluginId::Sound
    ));
    assert!(report.has_fatal_diagnostics());
    assert!(report
        .fatal_messages()
        .iter()
        .any(|diagnostic| diagnostic.contains("required runtime plugin Sound is unavailable")));
    assert!(report
        .required_missing_summary()
        .contains("required runtime plugin Sound is unavailable"));
    assert!(report
        .required_missing()
        .iter()
        .any(|missing| missing.runtime_id == RuntimePluginId::Sound));
}

#[test]
fn minimal_runtime_profile_load_report_has_structured_core_availability() {
    let report = runtime_modules_for_runtime_profile(RuntimeProfileId::Minimal);

    assert!(!report.has_fatal_diagnostics());
    assert!(report
        .runtime_plugin_availability
        .missing_required
        .is_empty());
    assert!(report
        .runtime_plugin_availability
        .externalized_missing
        .is_empty());
}

#[cfg(not(feature = "ui"))]
#[test]
fn required_ui_without_compiled_ui_feature_is_structured_missing() {
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Ui,
            true,
            true,
        )],
    };

    let report = runtime_modules_for_target(RuntimeTargetMode::ClientRuntime, Some(&manifest));

    assert!(report.runtime_plugin_availability.stub.iter().any(|entry| {
        entry.runtime_id == RuntimePluginId::Ui && entry.reason.contains("ui feature is disabled")
    }));
    assert!(report
        .required_missing()
        .iter()
        .any(|entry| entry.runtime_id == RuntimePluginId::Ui));
    assert!(report.has_fatal_diagnostics());
}
