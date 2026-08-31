use crate::ui::timeline_strip::TimelineStripGeneration;

use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::super::template_diamond_glyph::push_aa_diamond;
use super::geometry::TimelineStripGeometry;
use super::metrics::TimelineStripMetrics;
use super::palette::TimelineStripPalette;

pub(super) fn push_timeline_keys_and_playhead(
    commands: &mut Vec<HostPaintCommand>,
    generation: &TimelineStripGeneration,
    geometry: &TimelineStripGeometry,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    metrics: TimelineStripMetrics,
    palette: TimelineStripPalette,
) {
    let playhead_x = geometry.x_for_time(generation.current_time(), generation.duration());
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: playhead_x - metrics.playhead_width * 0.5,
            y: geometry.ruler.y + geometry.ruler.height * 0.48,
            width: metrics.playhead_width,
            height: (geometry.plot.y + geometry.plot.height
                - geometry.ruler.y
                - geometry.ruler.height * 0.48)
                .max(1.0),
        },
        Some(clip.clone()),
        order + 8,
        Some(palette.playhead),
        None,
        0.0,
        0.0,
        opacity,
    ));
    push_aa_diamond(
        commands,
        playhead_x,
        geometry.ruler.y + geometry.ruler.height * 0.48,
        metrics.key_radius,
        palette.playhead,
        clip,
        order + 9,
        opacity,
    );

    for key in generation.keys() {
        let x = geometry.x_for_time(key.time(), generation.duration());
        let y = geometry.track.y + geometry.track.height * 0.5;
        push_aa_diamond(
            commands,
            x,
            y,
            if key.selected() {
                metrics.key_radius + 1
            } else {
                metrics.key_radius
            },
            palette.key,
            clip,
            order + 10,
            opacity,
        );
        push_aa_diamond(
            commands,
            x,
            y,
            1,
            palette.key_center,
            clip,
            order + 11,
            opacity,
        );
    }
}
