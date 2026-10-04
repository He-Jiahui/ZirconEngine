use super::*;
use crate::ui::workbench::fixture::default_preview_fixture;
use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;

#[test]
fn authored_inspector_follows_the_resolved_right_shell_at_wide_and_compact_widths() {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let mut model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let tokens = EditorDesignTokens::workbench_dark();
    let metrics = WorkbenchChromeMetrics::default();
    for slot in [
        ActivityDrawerSlot::RightTop,
        ActivityDrawerSlot::RightBottom,
    ] {
        let drawer = model.drawer_ring.drawers.get_mut(&slot).unwrap();
        drawer.extent = tokens.density.right_drawer_width + metrics.rail_width;
        drawer.mode = ActivityDrawerMode::Pinned;
        drawer.visible = true;
    }
    for width in [1280.0, 900.0] {
        let size = UiSize::new(width, 800.0);
        let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(size).unwrap();
        bridge
            .recompute_layout_with_workbench_model(size, &model, &metrics)
            .unwrap();
        let shell = bridge.control_frame(RIGHT_DRAWER_SHELL_CONTROL_ID).unwrap();
        let inspector = bridge.control_frame("WorkbenchInspectorPanel").unwrap();
        assert!(shell.width > 0.0);
        assert!(inspector.width <= shell.width + 1.0, "Inspector must shrink with its compact drawer instead of keeping the token preferred width");
        for control in [
            "WorkbenchTransformPositionX",
            "WorkbenchTransformPositionY",
            "WorkbenchTransformPositionZ",
        ] {
            let field = bridge.control_frame(control).unwrap();
            assert!(field.width > 0.0);
            assert!(
                field.x >= shell.x - 1.0 && field.x + field.width <= shell.x + shell.width + 1.0,
                "every axis remains inside the horizontal viewport; vertical scroll owns overflow"
            );
        }
    }
    let size = UiSize::new(640.0, 800.0);
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(size).unwrap();
    bridge
        .recompute_layout_with_workbench_model(size, &model, &metrics)
        .unwrap();
    assert!(bridge
        .control_frame(RIGHT_DRAWER_SHELL_CONTROL_ID)
        .is_none());
}
