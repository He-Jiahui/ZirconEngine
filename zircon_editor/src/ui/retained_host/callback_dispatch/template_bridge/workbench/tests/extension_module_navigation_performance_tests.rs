use super::*;

#[test]
fn extension_navigation_uses_one_process_action_index() {
    let first = extension_action_index();
    let second = extension_action_index();

    assert!(std::ptr::eq(first, second));
    assert!(first.len() > 500);
    assert_eq!(
        workbench_extension_workspace_control_id("workbench.extension.terrain_editor.open"),
        Some("WorkbenchExtensionTerrainEditorWorkspace")
    );
    assert!(workbench_extension_panel_field_action(
        "workbench.extension.save_data.compression.edit"
    ));
}

#[test]
fn contentless_extension_tabs_are_absent_from_the_action_index() {
    assert!(!extension_action_index()
        .keys()
        .any(|action_id| action_id.ends_with("_tab.select")));
}
