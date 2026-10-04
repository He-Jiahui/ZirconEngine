use super::*;

#[test]
fn published_collapsed_bands_keep_their_geometry_and_authority() {
    let frame = UiFrame::new(0.0, 100.0, 900.0, 0.0);
    let layout = host_window_layout(BuiltinWorkbenchWindowLayoutFrames {
        center_band_frame: Some(UiFrame::new(0.0, 62.0, 900.0, 38.0)),
        document_region_frame: Some(UiFrame::new(46.0, 62.0, 854.0, 38.0)),
        bottom_region_frame: Some(frame),
        status_bar_frame: Some(frame),
        ..Default::default()
    });
    let published = to_host_contract_host_window_layout(&layout);
    assert!(published.authoritative);
    assert_eq!(published.status_bar_frame.y, 100.0);
    assert_eq!(published.status_bar_frame.width, 900.0);
    assert_eq!(published.status_bar_frame.height, 0.0);
    assert_eq!(published.bottom_region_frame.height, 0.0);
    assert!(!host_window_layout(Default::default()).authoritative);
}
