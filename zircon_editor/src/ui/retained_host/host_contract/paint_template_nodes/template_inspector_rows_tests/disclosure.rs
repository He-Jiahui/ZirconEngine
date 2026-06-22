use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::super::style::{disclosure_label_color, INSPECTOR_DISCLOSURE_LABEL_COLOR};
use super::support::{changed_pixel_count, inspector_node};
use crate::ui::layouts::common::model_rc;

#[test]
fn lighting_disclosure_row_paints_chevron_and_label_only() {
    let bytes = paint_template_nodes_for_test(
        220,
        42,
        model_rc(vec![inspector_node(
            "WorkbenchInspectorLightingRow",
            "Lighting",
            "",
        )]),
    );

    assert!(changed_pixel_count(&bytes, 220, 2, 12, 16, 16) > 0);
    assert_eq!(changed_pixel_count(&bytes, 220, 150, 10, 50, 20), 0);
    assert_eq!(disclosure_label_color(), INSPECTOR_DISCLOSURE_LABEL_COLOR);
}
