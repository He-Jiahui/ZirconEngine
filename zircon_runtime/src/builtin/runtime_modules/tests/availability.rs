use crate::builtin::{runtime_modules_for_runtime_profile, RuntimePluginId};
use crate::plugin::RuntimeProfileId;

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
        .effective_errors()
        .iter()
        .any(|diagnostic| diagnostic.contains("required runtime plugin Sound is unavailable")));
    assert!(report
        .required_missing_summary()
        .contains("required runtime plugin Sound is unavailable"));
    assert!(report
        .required_missing()
        .iter()
        .any(|missing| missing.id == RuntimePluginId::Sound));
    assert!(report
        .effective_required_missing()
        .iter()
        .any(|missing| missing.id == RuntimePluginId::Sound));
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
