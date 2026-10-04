use super::*;

#[test]
fn search_icon_preserves_fractional_post_dpi_origin() {
    let rect = FrameRect {
        x: 10.25,
        y: 20.5,
        width: 160.0,
        height: 31.25,
    };
    let metrics = workbench_field_metrics();

    let icon = search_icon_rect(&rect).expect("search icon frame");

    assert_eq!(icon.x, rect.x + metrics.input_pad_left);
    assert_eq!(
        icon.y,
        rect.y + (rect.height - metrics.search_icon_size).max(0.0) * 0.5
    );
    assert_ne!(icon.x.fract(), 0.0);
    assert_ne!(icon.y.fract(), 0.0);
}
