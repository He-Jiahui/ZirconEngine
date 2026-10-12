use super::*;

#[test]
fn search_icon_preserves_fractional_post_dpi_origin() {
    let rect = FrameRect {
        x: 10.25,
        y: 20.5,
        width: 160.0,
        height: 31.25,
    };
    let node = TemplatePaneNodeData::default();
    let metrics = workbench_field_metrics_for_node(&node);

    let icon = search_icon_rect(&node, &rect).expect("search icon frame");

    assert_eq!(icon.x, rect.x + metrics.input_pad_left);
    assert_eq!(
        icon.y,
        rect.y + (rect.height - metrics.search_icon_size).max(0.0) * 0.5
    );
    assert_ne!(icon.x.fract(), 0.0);
    assert_ne!(icon.y.fract(), 0.0);
}

#[test]
fn search_icon_and_text_share_authored_left_inset() {
    let node = TemplatePaneNodeData {
        authored_field_padding: [Some(0.0), None, None, None],
        ..Default::default()
    };
    let rect = FrameRect {
        x: 10.25,
        y: 20.5,
        width: 160.0,
        height: 31.25,
    };
    let icon = search_icon_rect(&node, &rect).expect("authored search icon frame");
    let metrics = workbench_field_metrics_for_node(&node);
    assert_eq!(icon.x, rect.x);
    assert!(metrics.search_text_left >= metrics.search_icon_size);
}
