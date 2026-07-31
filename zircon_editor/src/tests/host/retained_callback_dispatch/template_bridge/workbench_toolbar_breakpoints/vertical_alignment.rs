use super::*;

#[test]
fn toolbar_commands_and_dividers_center_within_their_rows() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };
    let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        FULL_WORKBENCH_WIDTH as f32,
        FULL_WORKBENCH_HEIGHT as f32,
    ))
    .expect("full workbench should build");

    for (group_id, children) in [
        (
            "WorkbenchToolbarFileGroup",
            &[
                "WorkbenchToolbarMenu",
                "WorkbenchToolbarNew",
                "WorkbenchToolbarOpen",
                "WorkbenchToolbarSave",
            ][..],
        ),
        (
            "WorkbenchModuleCommands",
            &[
                "WorkbenchModuleSave",
                "WorkbenchModuleBrowse",
                "WorkbenchModuleCompile",
                "WorkbenchModuleDiff",
                "WorkbenchModuleSimulate",
            ][..],
        ),
        (
            "WorkbenchToolbarToolGroup",
            &[
                "WorkbenchToolSelect",
                "WorkbenchToolMove",
                "WorkbenchToolRotate",
                "WorkbenchToolScale",
                "WorkbenchToolSnap",
            ][..],
        ),
        (
            "WorkbenchToolbarRunGroup",
            &[
                "WorkbenchRunPlay",
                "WorkbenchRunMode",
                "WorkbenchLayoutGrid",
                "WorkbenchThemeToggle",
            ][..],
        ),
    ] {
        for control_id in children {
            assert_vertically_centered(&bridge, group_id, control_id);
        }
    }

    for divider in [
        "WorkbenchToolbarFileGroupDivider",
        "WorkbenchToolbarToolGroupDivider",
        "WorkbenchToolbarRunGroupDivider",
    ] {
        assert_vertically_centered(&bridge, "WorkbenchToolbarCommandRow", divider);
    }
}

fn assert_vertically_centered(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    parent_id: &str,
    control_id: &str,
) {
    let parent = bridge
        .control_frame(parent_id)
        .unwrap_or_else(|| panic!("{parent_id} should have a frame"));
    let control = bridge
        .control_frame(control_id)
        .unwrap_or_else(|| panic!("{control_id} should have a frame"));
    assert_frame_value(
        &format!("{control_id} centered y"),
        control.y,
        parent.y + (parent.height - control.height) * 0.5,
    );
}
