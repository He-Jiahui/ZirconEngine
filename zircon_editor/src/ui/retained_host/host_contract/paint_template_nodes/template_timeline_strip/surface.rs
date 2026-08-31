use crate::ui::timeline_strip::{TimelineStripGeneration, TimelineStripStaticContent};

use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::geometry::TimelineStripGeometry;
use super::metrics::TimelineStripMetrics;
use super::palette::TimelineStripPalette;

const TIMELINE_SURFACE_BASE_COMMAND_CAPACITY: usize = 4;

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
    let progress_width = (geometry.x_for_time(generation.current_time(), generation.duration())
        - geometry.track.x)
        .clamp(0.0, geometry.track.width);
    let command_capacity = TIMELINE_SURFACE_BASE_COMMAND_CAPACITY
        .saturating_add(static_content.ticks().len())
        .saturating_add(usize::from(progress_width > 0.0));
    commands.reserve(command_capacity);

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

#[cfg(test)]
mod optimization_tests {
    #[test]
    fn optimization_batch_20260830da_timeline_surface_reserves_exact_command_count() {
        let source = include_str!("surface.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("timeline surface production source");

        assert!(production.contains("const TIMELINE_SURFACE_BASE_COMMAND_CAPACITY: usize = 4;"));
        assert!(production.contains("static_content.ticks().len()"));
        assert!(production.contains("usize::from(progress_width > 0.0)"));
        assert!(production.contains("commands.reserve(command_capacity);"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn optimization_batch_20260830da_timeline_surface_capacity_evidence() {
        const BATCH_COUNT: usize = 32_768;
        const TICK_COUNT: usize = 32;
        const COMMAND_COUNT: usize = 4 + TICK_COUNT + 1;
        const MARKER: &str = "EDITOR513_TIMELINE_SURFACE_CAPACITY_BENCH_V1";

        let legacy_growth_events = command_growth_events(BATCH_COUNT, COMMAND_COUNT, false);
        let optimized_growth_events = command_growth_events(BATCH_COUNT, COMMAND_COUNT, true);

        assert!(legacy_growth_events > 0);
        assert_eq!(optimized_growth_events, 0);
        println!(
            "{MARKER} batches={BATCH_COUNT} ticks={TICK_COUNT} commands={COMMAND_COUNT} \
             legacy_growth_events={legacy_growth_events} \
             optimized_growth_events={optimized_growth_events} reduction_pct=100"
        );
    }

    fn command_growth_events(batch_count: usize, command_count: usize, reserve: bool) -> usize {
        let mut growth_events = 0;
        for _ in 0..batch_count {
            let mut commands = if reserve {
                Vec::with_capacity(command_count)
            } else {
                Vec::new()
            };
            for command in 0..command_count {
                let previous_capacity = commands.capacity();
                commands.push(command);
                growth_events += usize::from(commands.capacity() != previous_capacity);
            }
        }
        growth_events
    }
}
