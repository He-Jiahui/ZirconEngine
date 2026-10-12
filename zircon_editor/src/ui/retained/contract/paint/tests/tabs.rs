use super::*;

#[test]
fn collapsed_tab_has_no_paint_label_or_underline_extent() {
    let tab = FrameRect {
        x: 12.0,
        y: 8.0,
        width: 0.0,
        height: 0.0,
    };

    let painted = tab_paint_rect(&TemplatePaneNodeData::default(), &tab);
    let label = tab_label_rect(&tab);
    let underline = tab_underline_rect(&tab);

    assert_eq!((painted.width, painted.height), (0.0, 0.0));
    assert_eq!((label.width, label.height), (0.0, 0.0));
    assert_eq!((underline.width, underline.height), (0.0, 0.0));
}

#[test]
fn non_finite_tab_extent_does_not_produce_a_non_finite_underline_origin() {
    let underline = tab_underline_rect(&FrameRect {
        x: 12.0,
        y: 8.0,
        width: f32::NAN,
        height: f32::INFINITY,
    });

    assert_eq!(underline.x, 12.0);
    assert_eq!(underline.y, 8.0);
    assert_eq!((underline.width, underline.height), (0.0, 0.0));
}
