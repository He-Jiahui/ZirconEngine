use crate::ui::layouts::common::model_rc;

use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::support::{changed_pixel_count, pixel_at, title_node};

#[test]
fn component_drawer_section_title_paints_bold_label() {
    let bytes = paint_template_nodes_for_test(
        180,
        48,
        model_rc(vec![title_node("WorkbenchButtonsTitle", "Buttons")]),
    );

    assert!(changed_pixel_count(&bytes, 180, 18, 14, 72, 20) > 0);
    assert_eq!(pixel_at(&bytes, 180, 12, 8), [0, 0, 0, 255]);
}

#[test]
fn inspector_section_title_paints_leading_icon_and_label() {
    let bytes = paint_template_nodes_for_test(
        180,
        48,
        model_rc(vec![title_node("WorkbenchInspectorTitle", "Props")]),
    );

    assert!(changed_pixel_count(&bytes, 180, 18, 17, 18, 18) > 0);
    assert!(changed_pixel_count(&bytes, 180, 43, 14, 58, 20) > 0);
}
