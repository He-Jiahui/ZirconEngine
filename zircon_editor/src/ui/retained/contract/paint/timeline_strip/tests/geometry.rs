use super::*;

#[test]
fn collapsed_timeline_strip_has_no_drawable_regions() {
    let geometry = TimelineStripGeometry::from_frame(
        &FrameRect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 120.0,
        },
        super::super::metrics::timeline_metrics(),
    );

    for region in [
        geometry.ruler,
        geometry.plot,
        geometry.track,
        geometry.footer,
    ] {
        assert_eq!(region.width, 0.0);
        assert_eq!(region.height, 0.0);
    }
}
