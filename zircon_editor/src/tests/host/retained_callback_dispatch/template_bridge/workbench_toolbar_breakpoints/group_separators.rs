use super::*;

const FILE_GROUP_SEPARATOR: &str = "WorkbenchToolbarFileGroupDivider";
const TOOL_GROUP_SEPARATOR: &str = "WorkbenchToolbarToolGroupDivider";
const RUN_GROUP_SEPARATOR: &str = "WorkbenchToolbarRunGroupDivider";
const LAYOUT_GROUP_SEPARATOR: &str = "WorkbenchToolbarLayoutGroupDivider";

#[test]
fn toolbar_group_dividers_follow_command_priority() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };

    for (width, tool_group_visible) in [
        (420.0, false),
        (640.0, false),
        (900.0, false),
        (1260.0, true),
        (1672.0, true),
    ] {
        let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(width, 720.0))
            .unwrap_or_else(|error| panic!("{width}px workbench should build: {error:?}"));

        for control_id in [FILE_GROUP_SEPARATOR, RUN_GROUP_SEPARATOR] {
            assert_eq!(
                control_visibility(&bridge, control_id),
                Some(UiVisibility::Visible),
                "{control_id} should keep primary command groups distinct at {width}px"
            );
            assert_vertical_divider_frame(&bridge, control_id);
        }

        assert_eq!(
            control_visibility(&bridge, TOOL_GROUP_SEPARATOR),
            Some(if tool_group_visible {
                UiVisibility::Visible
            } else {
                UiVisibility::Collapsed
            }),
            "tool divider should follow the transform tool group at {width}px"
        );
        if tool_group_visible {
            assert_vertical_divider_frame(&bridge, TOOL_GROUP_SEPARATOR);
        }

        assert_eq!(
            control_visibility(&bridge, LAYOUT_GROUP_SEPARATOR),
            Some(UiVisibility::Visible),
            "layout divider should keep iconized layout commands distinct at {width}px"
        );
        assert_vertical_divider_frame(&bridge, LAYOUT_GROUP_SEPARATOR);
    }
}

#[test]
fn toolbar_group_dividers_preserve_command_order() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };
    let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("full workbench should build");

    let file_group = bridge
        .control_frame("WorkbenchToolbarFileGroup")
        .expect("file group should have a frame");
    let file_divider = bridge
        .control_frame(FILE_GROUP_SEPARATOR)
        .expect("file divider should have a frame");
    let module_commands = bridge
        .control_frame("WorkbenchModuleCommands")
        .expect("module commands should have a frame");
    let left_spacer = bridge
        .control_frame("WorkbenchToolbarSpacerLeft")
        .expect("left toolbar spacer should have a frame");
    let tool_divider = bridge
        .control_frame(TOOL_GROUP_SEPARATOR)
        .expect("tool divider should have a frame");
    let tool_group = bridge
        .control_frame("WorkbenchToolbarToolGroup")
        .expect("tool group should have a frame");
    let right_spacer = bridge
        .control_frame("WorkbenchToolbarSpacerRight")
        .expect("right toolbar spacer should have a frame");
    let run_divider = bridge
        .control_frame(RUN_GROUP_SEPARATOR)
        .expect("run divider should have a frame");
    let run_group = bridge
        .control_frame("WorkbenchToolbarRunGroup")
        .expect("run group should have a frame");
    let layout_divider = bridge
        .control_frame(LAYOUT_GROUP_SEPARATOR)
        .expect("layout divider should have a frame");
    let layout_group = bridge
        .control_frame("WorkbenchToolbarLayoutGroup")
        .expect("layout group should have a frame");

    assert!(file_group.right() <= file_divider.x);
    assert!(file_divider.right() <= left_spacer.x);
    assert!(left_spacer.right() <= module_commands.x);
    assert!(module_commands.right() <= tool_divider.x);
    assert!(tool_divider.right() <= tool_group.x);
    assert!(tool_group.right() <= right_spacer.x);
    assert!(right_spacer.right() <= run_divider.x);
    assert!(run_divider.right() <= run_group.x);
    assert!(run_group.right() <= layout_divider.x);
    assert!(layout_divider.right() <= layout_group.x);
    assert!(
        left_spacer.width > 0.0,
        "left spacer should absorb free width"
    );
    assert!(
        right_spacer.width > 0.0,
        "right spacer should absorb free width"
    );
}

fn assert_vertical_divider_frame(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
) {
    let frame = bridge
        .control_frame(control_id)
        .unwrap_or_else(|| panic!("{control_id} should have a frame"));
    assert_frame_value(&format!("{control_id} width"), frame.width, 1.0);
    assert!(
        frame.height >= 20.0,
        "{control_id} should paint a readable vertical rule, got {}px",
        frame.height
    );
}
