use super::WORKBENCH_EXTENSION_PREVIEW_ACTION_IDS;

#[test]
fn extension_preview_action_domains_preserve_count_and_order() {
    assert_eq!(WORKBENCH_EXTENSION_PREVIEW_ACTION_IDS.len(), 754);
    assert_eq!(
        WORKBENCH_EXTENSION_PREVIEW_ACTION_IDS[0],
        "workbench.extension.shader_editor.open"
    );
    assert_eq!(
        WORKBENCH_EXTENSION_PREVIEW_ACTION_IDS[617],
        "workbench.extension.ui_binding.converter.commit"
    );
    assert_eq!(
        WORKBENCH_EXTENSION_PREVIEW_ACTION_IDS[618],
        "workbench.extension.icon_library.open"
    );
    assert_eq!(
        WORKBENCH_EXTENSION_PREVIEW_ACTION_IDS[753],
        "workbench.extension.telemetry_dashboard.segment.commit"
    );
}
