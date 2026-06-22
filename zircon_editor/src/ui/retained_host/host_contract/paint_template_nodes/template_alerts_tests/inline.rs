use super::super::super::super::paint_theme::PALETTE;
use super::super::super::style_selector::{
    WORKBENCH_ALERT_INFO_SURFACE, WORKBENCH_ALERT_WARNING_SURFACE,
};
use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::support::{changed_pixel_count, pixel_at, positioned_alert_node};
use crate::ui::layouts::common::model_rc;

#[test]
fn workbench_info_alert_paints_tinted_surface_icon_and_label() {
    let bytes = paint_template_nodes_for_test(
        192,
        48,
        model_rc(vec![positioned_alert_node(
            "WorkbenchInfoAlert",
            "Info Alert",
            "info",
            8.0,
            8.0,
            160.0,
            32.0,
        )]),
    );

    assert_eq!(pixel_at(&bytes, 192, 80, 24), WORKBENCH_ALERT_INFO_SURFACE);
    assert_eq!(pixel_at(&bytes, 192, 25, 24), PALETTE.info);
    assert!(changed_pixel_count(&bytes, 192, 38, 16, 62, 18) > 0);
    assert_eq!(pixel_at(&bytes, 192, 176, 24), [0, 0, 0, 255]);
}

#[test]
fn workbench_warning_alert_uses_warning_tone() {
    let bytes = paint_template_nodes_for_test(
        192,
        48,
        model_rc(vec![positioned_alert_node(
            "WorkbenchWarningAlert",
            "Warning Alert",
            "warning",
            8.0,
            8.0,
            160.0,
            32.0,
        )]),
    );

    assert_eq!(
        pixel_at(&bytes, 192, 150, 24),
        WORKBENCH_ALERT_WARNING_SURFACE
    );
    assert_eq!(pixel_at(&bytes, 192, 27, 18), PALETTE.warning);
    assert!(changed_pixel_count(&bytes, 192, 38, 16, 84, 18) > 0);
}
