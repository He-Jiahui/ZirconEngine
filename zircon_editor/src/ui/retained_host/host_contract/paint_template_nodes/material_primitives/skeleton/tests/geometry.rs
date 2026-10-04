use super::*;
use crate::ui::retained_host::host_contract::paint_theme::METRICS;

#[test]
fn skeleton_default_radius_tracks_host_control_density() {
    let node = TemplatePaneNodeData::default();
    let rect = FrameRect {
        x: 4.0,
        y: 8.0,
        width: 40.0,
        height: 16.0,
    };
    let mut compact = METRICS;
    compact.radius_control = 3.0;

    assert_eq!(skeleton_corner_radius_from_host(&node, &rect, compact), 3.0);
}

#[test]
fn skeleton_frames_stay_inside_tight_parent_bounds() {
    let rect = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 0.4,
        height: 0.6,
    };
    let mut circular = TemplatePaneNodeData::default();
    circular.component_variant = "circular".to_owned();
    let mut text = TemplatePaneNodeData::default();
    text.component_variant = "text".to_owned();

    for frame in [
        skeleton_frame_for_variant(&circular, &rect),
        skeleton_frame_for_variant(&text, &rect),
        skeleton_wave_frame(&rect),
    ] {
        assert!(frame.x >= rect.x);
        assert!(frame.y >= rect.y);
        assert!(frame.right() <= rect.right());
        assert!(frame.bottom() <= rect.bottom());
    }
}

#[test]
fn skeleton_radius_does_not_exceed_narrow_frame_bounds() {
    let rect = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 2.0,
        height: 20.0,
    };

    assert_eq!(
        skeleton_corner_radius_from_host(&TemplatePaneNodeData::default(), &rect, METRICS),
        1.0
    );
}
