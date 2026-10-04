use super::*;

#[test]
fn welcome_main_column_fallback_metrics_derive_from_shared_spacing_and_rows() {
    let panel = FrameRect {
        x: 100.0,
        y: 20.0,
        width: 736.0,
        height: 520.0,
    };

    let welcome = welcome_main_column_frame_metrics(&panel);

    let metrics = current_host_metrics();
    let expected_inset = metrics.gap_l + metrics.gap_m * 2.0;
    assert_eq!(welcome.content_x, panel.x + expected_inset);
    assert_eq!(welcome.content_width, WELCOME_CONTENT_MAX_WIDTH);
    assert_eq!(welcome.top_inset, expected_inset);
    assert_eq!(welcome.section_gap, metrics.gap_l);
    assert_eq!(welcome.hero_height, metrics.row_height * 3.0);
    assert_eq!(
        welcome.status_height,
        metrics.row_height + metrics.border_width * 2.0
    );
    assert_eq!(welcome.header_height, metrics.row_height + metrics.gap_m);
}
