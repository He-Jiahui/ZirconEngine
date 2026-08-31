use crate::ui::weight_heatmap::WeightHeatmapSource;

use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::geometry::WeightHeatmapGeometry;
use super::palette::{SELECTED_SOURCE, SOURCE_MARKER};

pub(super) fn push_heat_source_markers(
    commands: &mut Vec<HostPaintCommand>,
    sources: &[WeightHeatmapSource],
    geometry: &WeightHeatmapGeometry,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if !geometry.is_drawable() {
        return;
    }
    commands.reserve(sources.len());
    for source in sources {
        let x = geometry.x_for_normalized(source.x());
        let y = geometry.y_for_normalized(source.y());
        push_source_marker(
            commands,
            x,
            y,
            if source.selected() { 5.0 } else { 3.0 },
            if source.selected() {
                SELECTED_SOURCE
            } else {
                SOURCE_MARKER
            },
            clip,
            order + 4,
            opacity,
        );
    }
}

fn push_source_marker(
    commands: &mut Vec<HostPaintCommand>,
    x: f32,
    y: f32,
    radius: f32,
    color: [u8; 4],
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: x - radius,
            y: y - radius,
            width: radius * 2.0 + 1.0,
            height: radius * 2.0 + 1.0,
        },
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        radius,
        opacity,
    ));
}

#[cfg(test)]
mod tests {
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
}

#[cfg(test)]
#[path = "markers/capacity_tests.rs"]
mod capacity_tests;
