use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::geometry::TimelineStripGeometry;
use super::metrics::TimelineStripMetrics;
use super::palette::TimelineStripPalette;

pub(super) const MAX_TIMELINE_TICKS: usize = 4_096;

pub(super) fn push_timeline_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    geometry: &TimelineStripGeometry,
    ticks: &[f32],
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

    for &tick in ticks {
        let x = geometry.x_for_time(tick, node.timeline_strip.duration);
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
    let progress_width = (geometry.x_for_time(
        node.timeline_strip.current_time,
        node.timeline_strip.duration,
    ) - geometry.track.x)
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

pub(super) fn timeline_ticks(duration: f32, interval: f32, max_ticks: usize) -> Vec<f32> {
    if !duration.is_finite() || !interval.is_finite() || duration <= 0.0 || interval <= 0.0 {
        return vec![0.0, 1.0];
    }

    let max_ticks = max_ticks.clamp(2, MAX_TIMELINE_TICKS);
    let requested_segments = (duration / interval).ceil();
    let segment_budget = max_ticks - 1;
    let segment_count =
        if !requested_segments.is_finite() || requested_segments >= segment_budget as f32 {
            segment_budget
        } else {
            (requested_segments as usize).max(1)
        };
    let step = if requested_segments > segment_count as f32 {
        duration / segment_count as f32
    } else {
        interval
    };
    let mut ticks = Vec::with_capacity(segment_count + 1);
    for index in 0..segment_count {
        ticks.push(index as f32 * step);
    }
    ticks.push(duration);
    ticks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeline_ticks_are_bounded_and_keep_the_duration_endpoint() {
        let ticks = timeline_ticks(10.0, f32::MIN_POSITIVE, 128);

        assert_eq!(ticks.len(), 128);
        assert_eq!(ticks.first().copied(), Some(0.0));
        assert_eq!(ticks.last().copied(), Some(10.0));
    }

    #[test]
    fn timeline_ticks_respect_smaller_visual_budget() {
        let ticks = timeline_ticks(3.0, 0.5, 4);

        assert_eq!(ticks, vec![0.0, 1.0, 2.0, 3.0]);
    }

    #[test]
    fn timeline_ticks_keep_requested_interval_within_budget() {
        let ticks = timeline_ticks(3.0, 0.5, 32);

        assert_eq!(ticks, vec![0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0]);
    }
}
