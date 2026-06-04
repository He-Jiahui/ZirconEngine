use super::super::super::*;

#[test]
fn sound_plugin_registration_contributes_optional_timeline_dependency() {
    let report = RuntimePluginRegistrationReport::from_plugin(&runtime_plugin());

    assert!(report
        .package_manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.id == "timeline_sequence" && !dependency.required));
}
