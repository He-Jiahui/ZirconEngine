use super::*;
use crate::ui::retained_host::host_contract::paint_theme::METRICS;

#[test]
fn rounded_avatar_radius_tracks_host_control_density_and_frame_bounds() {
    let mut node = TemplatePaneNodeData::default();
    node.component_variant = "rounded".to_owned();
    let mut compact = METRICS;
    compact.radius_control = 3.0;
    let wide = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 32.0,
        height: 32.0,
    };
    let narrow = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 2.0,
        height: 8.0,
    };

    assert_eq!(avatar_corner_radius_from_host(&node, &wide, compact), 3.0);
    assert_eq!(avatar_corner_radius_from_host(&node, &narrow, compact), 1.0);
}
