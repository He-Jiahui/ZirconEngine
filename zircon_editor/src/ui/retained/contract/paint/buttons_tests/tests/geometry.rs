use super::super::button_paint_rect;
use super::support::{positioned_button_node, TemplatePaneNodeDataTestExt};

#[test]
fn workbench_button_preserves_fractional_frame_and_honors_declared_layout_offset() {
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

    assert!((rect.x - 15.4).abs() < 0.00001);
    assert!((rect.y - 7.4).abs() < 0.00001);
    assert_eq!(rect.width, 80.0);
    assert_eq!(rect.height, 32.0);
}
