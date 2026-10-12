use crate::ui::layouts::common::model_rc;

use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::support::{changed_pixel_count, label_node};

#[test]
fn scale_link_label_paints_link_glyph_without_text_fallback() {
    let bytes = paint_template_nodes_for_test(
        48,
        40,
        model_rc(vec![label_node("WorkbenchTransformScaleLink", "")]),
    );

    assert!(changed_pixel_count(&bytes, 48, 12, 14, 20, 12) > 0);
    assert_eq!(changed_pixel_count(&bytes, 48, 34, 8, 8, 24), 0);
}

#[test]
fn transform_axis_label_paints_compact_axis_text() {
    let bytes = paint_template_nodes_for_test(
        48,
        40,
        model_rc(vec![label_node("WorkbenchTransformRotationAxisY", "Y")]),
    );

    assert!(changed_pixel_count(&bytes, 48, 8, 10, 14, 20) > 0);
    assert_eq!(changed_pixel_count(&bytes, 48, 28, 10, 12, 20), 0);
}
