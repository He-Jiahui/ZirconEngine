use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::geometry::TimelineStripGeometry;
use super::metrics::TimelineStripMetrics;
use super::palette::TimelineStripPalette;
use super::surface::timeline_ticks;

pub(super) fn push_timeline_text(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    geometry: &TimelineStripGeometry,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    metrics: TimelineStripMetrics,
    palette: TimelineStripPalette,
) {
    for tick in timeline_ticks(
        node.timeline_strip.duration,
        node.timeline_strip.tick_interval,
    ) {
        let x = geometry.x_for_time(tick, node.timeline_strip.duration);
        push_text(
            commands,
            FrameRect {
                x: (x - metrics.line_height).max(geometry.ruler.x),
                y: geometry.ruler.y + metrics.inset,
                width: metrics.line_height * 2.0,
                height: metrics.line_height,
            },
            clip,
            order + 6,
            format_time(tick),
            palette.tick_text,
            metrics,
            opacity,
        );
    }

    if !node.timeline_strip.track_label.trim().is_empty() {
        push_text(
            commands,
            FrameRect {
                x: geometry.track.x + metrics.inset * 2.0,
                y: geometry.track.y + (geometry.track.height - metrics.line_height) * 0.5,
                width: (geometry.track.width * 0.45).max(1.0),
                height: metrics.line_height,
            },
            clip,
            order + 7,
            node.timeline_strip.track_label.to_string(),
            palette.track_text,
            metrics,
            opacity,
        );
    }

    let footer_text = format!(
        "{:.2} / {:.2} ({:.0}%)",
        node.timeline_strip.current_time,
        node.timeline_strip.duration,
        (node.timeline_strip.current_time / node.timeline_strip.duration.max(f32::EPSILON) * 100.0)
            .clamp(0.0, 100.0)
    );
    push_text(
        commands,
        FrameRect {
            x: geometry.footer.x + geometry.footer.width * 0.52,
            y: geometry.footer.y + (geometry.footer.height - metrics.line_height) * 0.5,
            width: geometry.footer.width * 0.46,
            height: metrics.line_height,
        },
        clip,
        order + 7,
        footer_text,
        palette.tick_text,
        metrics,
        opacity,
    );
}

fn push_text(
    commands: &mut Vec<HostPaintCommand>,
    frame: FrameRect,
    clip: &FrameRect,
    order: i32,
    text: String,
    color: [u8; 4],
    metrics: TimelineStripMetrics,
    opacity: f32,
) {
    commands.push(HostPaintCommand::text(
        frame,
        Some(clip.clone()),
        order,
        text,
        color,
        metrics.font_size,
        metrics.line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn format_time(time: f32) -> String {
    format!("{time:.1}")
}
