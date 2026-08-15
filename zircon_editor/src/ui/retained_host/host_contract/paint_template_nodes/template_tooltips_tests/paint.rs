use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::primitives::Color;

use super::super::super::style_selector::{
    select_workbench_tooltip_style, WORKBENCH_TOOLTIP_BORDER, WORKBENCH_TOOLTIP_SURFACE,
};
use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::super::super::template_tooltip_glyphs::tooltip_arrow_size;
use super::support::{changed_pixel_count, pixel_at, tooltip_node};

#[test]
fn workbench_tooltip_paints_declared_bubble_arrow_and_info_icon() {
    let mut node = tooltip_node();
    node.value_number = 8.0;
    node.value_color = Color::from_rgb_u8(23, 28, 32);
    node.label_color = Color::from_rgb_u8(168, 179, 184);
    node.icon_color = Color::from_rgb_u8(37, 156, 167);
    let declared_arrow = [23, 28, 32, 255];
    let declared_body = [168, 179, 184, 255];
    let declared_icon = [37, 156, 167, 255];

    let style = select_workbench_tooltip_style(&node);
    assert_eq!(tooltip_arrow_size(&node), 8.0);
    assert_eq!(style.arrow, declared_arrow);
    assert_eq!(style.body, declared_body);
    assert_eq!(style.icon, declared_icon);

    let bytes = paint_template_nodes_for_test(128, 96, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 128, 64, 12), WORKBENCH_TOOLTIP_SURFACE);
    assert_eq!(pixel_at(&bytes, 128, 64, 8), WORKBENCH_TOOLTIP_BORDER);
    assert!(
        changed_pixel_count(&bytes, 128, 59, 56, 10, 10) > 0,
        "tooltip arrow should paint below the bubble"
    );
    assert_eq!(pixel_at(&bytes, 128, 63, 69), declared_icon);
    assert!(changed_pixel_count(&bytes, 128, 22, 14, 50, 14) > 0);
    assert!(changed_pixel_count(&bytes, 128, 22, 29, 72, 14) > 0);
}
