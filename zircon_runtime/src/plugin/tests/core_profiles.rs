use super::EditorCoreProfile;

#[test]
fn editor_profile_reports_the_runtime_highlight_input_capability() {
    assert!(EditorCoreProfile::minimal()
        .required_capabilities
        .iter()
        .any(|capability| capability == "runtime.editor_overlay.highlight_set"));
}
