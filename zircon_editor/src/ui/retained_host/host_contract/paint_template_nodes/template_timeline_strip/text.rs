use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_text::measure_runtime_text_width;
use super::super::render_commands::HostPaintCommand;
use super::geometry::TimelineStripGeometry;
use super::metrics::TimelineStripMetrics;
use super::palette::TimelineStripPalette;

pub(super) fn push_timeline_text(
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
    for (index, &tick) in ticks.iter().enumerate() {
        let x = geometry.x_for_time(tick, node.timeline_strip.duration);
        let previous_x = index
            .checked_sub(1)
            .and_then(|previous| ticks.get(previous))
            .map(|previous| geometry.x_for_time(*previous, node.timeline_strip.duration));
        let next_x = ticks
            .get(index + 1)
            .map(|next| geometry.x_for_time(*next, node.timeline_strip.duration));
        let label = format_time(tick);
        push_text(
            commands,
            timeline_tick_label_frame(&geometry.ruler, x, previous_x, next_x, &label, metrics),
            clip,
            order + 6,
            label,
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

fn timeline_tick_label_frame(
    ruler: &FrameRect,
    tick_x: f32,
    previous_x: Option<f32>,
    next_x: Option<f32>,
    label: &str,
    metrics: TimelineStripMetrics,
) -> FrameRect {
    let ruler_right = ruler.x + ruler.width.max(1.0);
    let left_limit = previous_x
        .map(|previous| (previous + tick_x) * 0.5)
        .unwrap_or(ruler.x)
        .clamp(ruler.x, ruler_right);
    let right_limit = next_x
        .map(|next| (next + tick_x) * 0.5)
        .unwrap_or(ruler_right)
        .min(ruler_right)
        .max(left_limit);
    let width = measure_runtime_text_width(label, metrics.font_size)
        .ceil()
        .clamp(1.0, (right_limit - left_limit).max(1.0));
    let x = (tick_x - width * 0.5).clamp(left_limit, (right_limit - width).max(left_limit));

    FrameRect {
        x,
        y: ruler.y + metrics.inset,
        width,
        height: metrics.line_height,
    }
}

fn format_time(time: f32) -> String {
    format!("{time:.1}")
}

#[cfg(test)]
mod tests {
    use super::{TimelineStripMetrics, timeline_tick_label_frame};
    use crate::ui::retained_host::host_contract::{
        data::FrameRect, paint_text::measure_runtime_text_width,
    };

    fn metrics() -> TimelineStripMetrics {
        TimelineStripMetrics {
            outer_radius: 2.0,
            border_width: 1.0,
            inset: 2.0,
            ruler_height: 20.0,
            track_height: 20.0,
            footer_height: 20.0,
            font_size: 12.0,
            line_height: 16.0,
            key_radius: 3,
            playhead_width: 2.0,
        }
    }

    #[test]
    fn timeline_tick_label_frame_measures_and_clamps_to_neighbor_interval() {
        let ruler = FrameRect {
            x: 0.0,
            y: 4.0,
            width: 120.0,
            height: 20.0,
        };
        let label = "WWWWWWWWWWWW";
        let frame =
            timeline_tick_label_frame(&ruler, 60.0, Some(0.0), Some(120.0), label, metrics());

        let available_width = 60.0;
        let expected_width = measure_runtime_text_width(label, 12.0)
            .ceil()
            .clamp(1.0, available_width);
        assert_eq!(frame.width, expected_width);
        assert!(frame.x >= 30.0);
        assert!(frame.x + frame.width <= 90.0);
    }
}
