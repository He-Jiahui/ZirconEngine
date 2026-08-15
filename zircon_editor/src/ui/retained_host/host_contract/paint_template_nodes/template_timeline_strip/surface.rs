use crate::ui::timeline_strip::{TimelineStripGeneration, TimelineStripStaticContent};

use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::geometry::TimelineStripGeometry;
use super::metrics::TimelineStripMetrics;
use super::palette::TimelineStripPalette;

pub(super) fn push_timeline_surface(
    commands: &mut Vec<HostPaintCommand>,
    generation: &TimelineStripGeneration,
    geometry: &TimelineStripGeometry,
    static_content: &TimelineStripStaticContent,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    metrics: TimelineStripMetrics,
    palette: TimelineStripPalette,
) {
    commands.push(HostPaintCommand::quad(
        geometry.outer.clone(),
        Some(clip.clone()),
        order,
        Some(palette.outer_surface),
        Some(palette.outer_border),
        metrics.border_width,
        metrics.outer_radius,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        geometry.plot.clone(),
        Some(clip.clone()),
        order + 1,
        Some(palette.plot_surface),
        Some(palette.grid_line),
        metrics.border_width,
        0.0,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        geometry.footer.clone(),
        Some(clip.clone()),
        order + 2,
        Some(palette.footer_surface),
        Some(palette.grid_line),
        metrics.border_width,
        metrics.outer_radius,
        opacity,
    ));

    for tick in static_content.ticks() {
        let x = geometry.x_for_time(tick.value(), generation.duration());
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x,
                y: geometry.ruler.y + geometry.ruler.height * 0.58,
                width: metrics.border_width,
                height: (geometry.plot.y + geometry.plot.height
                    - geometry.ruler.y
                    - geometry.ruler.height * 0.58)
                    .max(1.0),
            },
            Some(clip.clone()),
            order + 3,
            Some(palette.grid_line),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }

    commands.push(HostPaintCommand::quad(
        geometry.track.clone(),
        Some(clip.clone()),
        order + 4,
        Some(palette.track_surface),
        None,
        0.0,
        0.0,
        opacity,
    ));
    let progress_width = (geometry.x_for_time(generation.current_time(), generation.duration())
        - geometry.track.x)
        .clamp(0.0, geometry.track.width);
    if progress_width > 0.0 {
        commands.push(HostPaintCommand::quad(
            FrameRect {
                width: progress_width,
                ..geometry.track.clone()
            },
            Some(clip.clone()),
            order + 5,
            Some(palette.track_progress),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }
}
