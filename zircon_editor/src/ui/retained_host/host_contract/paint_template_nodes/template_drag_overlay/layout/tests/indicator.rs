use super::*;

fn metrics() -> DragOverlayMetrics {
    DragOverlayMetrics {
        border_width: 1.0,
        preview_radius: 4.0,
        icon_radius: 4.0,
        font_size: 13.33,
        line_height: 16.0,
        icon_left: 12.0,
        icon_size: 16.0,
        text_left_with_icon: 35.0,
        text_right_inset: 12.0,
        indicator_thickness: 2.0,
    }
}

#[test]
fn indicator_scales_into_a_small_drop_target() {
    let node = TemplatePaneNodeData {
        has_drop_target: true,
        drop_target_x: 6.0,
        drop_target_y: 10.0,
        drop_target_width: 1.0,
        drop_target_height: 1.0,
        drop_indicator_edge: "bottom".into(),
        ..TemplatePaneNodeData::default()
    };
    let indicator = indicator_frame(&node, &metrics()).expect("small target has an indicator");

    assert_eq!(indicator.x, 6.0);
    assert_eq!(indicator.y, 10.0);
    assert_eq!(indicator.width, 1.0);
    assert_eq!(indicator.height, 1.0);
}

#[test]
fn indicator_skips_a_collapsed_drop_target() {
    let node = TemplatePaneNodeData {
        has_drop_target: true,
        drop_target_width: 0.0,
        drop_target_height: 4.0,
        drop_indicator_edge: "left".into(),
        ..TemplatePaneNodeData::default()
    };

    assert_eq!(indicator_frame(&node, &metrics()), None);
}
