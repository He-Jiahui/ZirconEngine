use super::super::button_paint_rect;
use super::support::{positioned_button_node, TemplatePaneNodeDataTestExt};

#[test]
fn workbench_button_honors_declared_layout_offset() {
    let mut node = positioned_button_node(
        "WorkbenchPrimaryButton",
        "Primary",
        "filled",
        12.4,
        8.4,
        80.0,
        32.0,
    );
    node.layout_offset_x = 3.0;
    node.layout_offset_y = -1.0;

    let rect = button_paint_rect(&node, &node.frame_rect());

    assert_eq!(rect.x, 15.0);
    assert_eq!(rect.y, 7.0);
    assert_eq!(rect.width, 80.0);
    assert_eq!(rect.height, 32.0);
}
