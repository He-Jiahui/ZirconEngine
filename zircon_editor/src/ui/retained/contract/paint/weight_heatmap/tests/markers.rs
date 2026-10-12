use super::*;
use crate::ui::retained_host::host_contract::data::FrameRect;

#[test]
fn each_source_marker_uses_one_paint_command() {
    let mut commands = Vec::new();
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 20.0,
        height: 20.0,
    };

    push_source_marker(&mut commands, 10.0, 10.0, 5.0, SOURCE_MARKER, &clip, 0, 1.0);

    assert_eq!(commands.len(), 1);
}

#[test]
fn collapsed_heatmap_does_not_emit_source_markers() {
    let geometry = WeightHeatmapGeometry::from_frame(
        &FrameRect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 32.0,
        },
        20.0,
    );
    let source = WeightHeatmapSource::new(0.5, 0.5, 1.0, false);
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 32.0,
        height: 32.0,
    };
    let mut commands = Vec::new();

    push_heat_source_markers(&mut commands, &[source], &geometry, &clip, 0, 1.0);

    assert!(commands.is_empty());
}
