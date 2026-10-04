// 静态刻度由 generation 按 plot 像素预算缓存；此层仅追加画布、轨道、进度表面，文字和关键帧分别由后续消费者处理。
use crate::ui::timeline_strip::{TimelineStripGeneration, TimelineStripStaticContent};

use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::geometry::TimelineStripGeometry;
use super::metrics::TimelineStripMetrics;
use super::palette::TimelineStripPalette;

const TIMELINE_SURFACE_BASE_COMMAND_CAPACITY: usize = 4;

/// 入参 static_content 应由同一 generation 和 geometry.plot.width 获取；只预留此表面层的命令容量。
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
#[path = "tests/surface_optimization_tests.rs"]
mod optimization_tests;
