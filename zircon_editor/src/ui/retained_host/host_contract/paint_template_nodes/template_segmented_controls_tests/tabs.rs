use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::super::super::template_segmented_control_geometry::tab_paint_rect;
use super::super::identity::is_workbench_tab;
use super::super::style::tab_background;
use super::support::{changed_pixel_count, frame_rect, pixel_at, tab_node};
use crate::ui::layouts::common::model_rc;

#[test]
fn selected_tab_paints_accent_underline_without_filling_right_edge() {
    let bytes = paint_template_nodes_for_test(180, 48, model_rc(vec![tab_node()]));

    assert!(changed_pixel_count(&bytes, 180, 0, 40, 150, 4) > 0);
    assert_eq!(pixel_at(&bytes, 180, 148, 8), [0, 0, 0, 255]);
}

#[test]
fn selected_tab_honors_declared_layout_offset() {
    let mut node = tab_node();
    node.control_id = "WorkbenchLabsTabOne".into();
    node.layout_offset_x = 3.0;
    node.layout_offset_y = 2.0;
    let paint_rect = tab_paint_rect(&node, &frame_rect(&node.frame));

    assert!(is_workbench_tab(&node));
    assert_eq!(paint_rect.x, 3.0);
    assert_eq!(paint_rect.y, 6.0);

    let bytes = paint_template_nodes_for_test(180, 52, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 180, 0, 44), [0, 0, 0, 255]);
    assert!(changed_pixel_count(&bytes, 180, 3, 44, 150, 2) > 0);
}

#[test]
fn workbench_tab_uses_declared_idle_background() {
    use zircon_runtime_interface::ui::style::{
        ResolvedButtonStyle, UiResolvedElementStyle, UiRgbaColor, UiStyleColor,
    };

    let mut node = tab_node();
    node.control_id = "WorkbenchLabsTabs".into();
    node.text = "".into();
    node.checked = false;
    node.selected = false;
    node.button_style = ResolvedButtonStyle {
        element: UiResolvedElementStyle {
            background_color: Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(28, 34, 38, 255))),
            ..UiResolvedElementStyle::default()
        },
        ..ResolvedButtonStyle::default()
    };

    assert!(is_workbench_tab(&node));
    assert_eq!(tab_background(&node), Some([28, 34, 38, 255]));

    let bytes = paint_template_nodes_for_test(180, 52, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 180, 8, 12), [28, 34, 38, 255]);
}
