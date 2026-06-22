use super::super::super::style_selector::{
    WORKBENCH_TOAST_ACTION, WORKBENCH_TOAST_BORDER, WORKBENCH_TOAST_SURFACE,
};
use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::super::{select_workbench_toast_style, toast_status_mark_size};
use super::support::{blend_over, changed_pixel_count, pixel_at, positioned_alert_node};
use crate::ui::layouts::common::model_rc;

#[test]
fn workbench_toast_paints_status_mark_action_and_close() {
    let bytes = paint_template_nodes_for_test(
        320,
        48,
        model_rc(vec![positioned_alert_node(
            "WorkbenchToastRoot",
            "Operation completed successfully",
            "success",
            8.0,
            8.0,
            280.0,
            32.0,
        )]),
    );

    let surface_pixel = blend_over(WORKBENCH_TOAST_SURFACE, [0, 0, 0, 255]);
    assert_eq!(WORKBENCH_TOAST_SURFACE, [21, 48, 53, 247]);
    assert_eq!(WORKBENCH_TOAST_BORDER, [53, 199, 208, 20]);
    assert_eq!(pixel_at(&bytes, 320, 160, 12), surface_pixel);
    assert_eq!(
        pixel_at(&bytes, 320, 160, 8),
        blend_over(WORKBENCH_TOAST_BORDER, surface_pixel)
    );
    assert_ne!(pixel_at(&bytes, 320, 120, 24), [0, 0, 0, 255]);
    assert_eq!(pixel_at(&bytes, 320, 35, 24), WORKBENCH_TOAST_ACTION);
    assert!(changed_pixel_count(&bytes, 320, 233, 16, 34, 18) > 0);
    assert!(changed_pixel_count(&bytes, 320, 269, 17, 12, 14) > 0);
}

#[test]
fn workbench_toast_uses_declared_status_mark_and_action_style() {
    let mut node = positioned_alert_node(
        "WorkbenchToastRoot",
        "Operation completed successfully",
        "success",
        8.0,
        8.0,
        280.0,
        32.0,
    );
    node.value_number = 12.0;
    node.label_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(32, 159, 169);
    node.value_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(35, 143, 152);

    let style = select_workbench_toast_style(&node);
    assert_eq!(toast_status_mark_size(&node), 12.0);
    assert_eq!(style.mark, [32, 159, 169, 255]);
    assert_eq!(style.action, [35, 143, 152, 255]);

    let bytes = paint_template_nodes_for_test(320, 48, model_rc(vec![node]));
    assert_eq!(pixel_at(&bytes, 320, 22, 20), [32, 159, 169, 255]);
}
