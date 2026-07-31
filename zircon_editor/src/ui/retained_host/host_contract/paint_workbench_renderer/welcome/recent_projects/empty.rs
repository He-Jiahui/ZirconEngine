use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_text::{
    draw_text_with_size_and_style, measure_runtime_text_width,
};
use super::super::super::super::paint_theme::{current_host_metrics, HostControlMetrics};
use super::super::style::WELCOME_MUTED_TEXT;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const MIN_EMPTY_TEXT_SLOT_WIDTH: f32 = 1.0;

pub(super) fn draw_recent_projects_empty_state(
    frame: &mut HostRgbaFrame,
    list: &FrameRect,
    clip: &FrameRect,
) {
    let metrics = current_host_metrics();
    let title_font_size = metrics.font_body;
    let title_line_height = shared_line_height(title_font_size, metrics);
    let subtitle_font_size = metrics.font_small;
    let subtitle_line_height = shared_line_height(subtitle_font_size, metrics);
    let content_height = title_line_height + metrics.gap_s + subtitle_line_height;
    let content_y = list.y + ((list.height - content_height).max(0.0) * 0.5);

    draw_centered_empty_text(
        frame,
        list,
        clip,
        content_y,
        "No recent projects",
        title_font_size,
        title_line_height,
        metrics,
    );
    draw_centered_empty_text(
        frame,
        list,
        clip,
        content_y + title_line_height + metrics.gap_s,
        "Create a new project to start",
        subtitle_font_size,
        subtitle_line_height,
        metrics,
    );
}

fn shared_line_height(font_size: f32, metrics: HostControlMetrics) -> f32 {
    metrics.line_height(font_size).round().max(font_size.ceil())
}

fn draw_centered_empty_text(
    frame: &mut HostRgbaFrame,
    list: &FrameRect,
    clip: &FrameRect,
    y: f32,
    text: &str,
    font_size: f32,
    line_height: f32,
    metrics: HostControlMetrics,
) {
    let available_width = (list.width - metrics.gap_l * 2.0).max(MIN_EMPTY_TEXT_SLOT_WIDTH);
    let text_width = (measure_runtime_text_width(text, font_size) + metrics.text_clip_guard)
        .min(available_width)
        .max(MIN_EMPTY_TEXT_SLOT_WIDTH);
    draw_text_with_size_and_style(
        frame,
        FrameRect {
            x: list.x + ((list.width - text_width).max(0.0) * 0.5),
            y,
            width: text_width,
            height: line_height,
        },
        text,
        Some(clip),
        WELCOME_MUTED_TEXT,
        font_size,
        line_height,
        UiTextRunPaintStyle::default(),
    );
}

#[cfg(test)]
mod tests {
    use super::super::super::super::super::paint_frame::HostRecordedPaintKind;
    use super::*;

    #[test]
    fn recent_empty_state_uses_shared_typography_and_centers_content_in_list() {
        let mut frame = HostRgbaFrame::recording_only(320, 180);
        let list = FrameRect {
            x: 12.0,
            y: 8.0,
            width: 296.0,
            height: 164.0,
        };
        let clip = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 320.0,
            height: 180.0,
        };

        draw_recent_projects_empty_state(&mut frame, &list, &clip);

        let commands = frame.into_recorded_commands();
        assert_eq!(commands.len(), 2);
        let metrics = current_host_metrics();
        for (command, expected_font_size) in
            commands.iter().zip([metrics.font_body, metrics.font_small])
        {
            let command_center_x = command.frame.x + command.frame.width * 0.5;
            assert!((command_center_x - (list.x + list.width * 0.5)).abs() <= 1.0);
            match &command.kind {
                HostRecordedPaintKind::Text { font_size, .. } => {
                    assert_eq!(*font_size, expected_font_size);
                }
                kind => panic!("recent empty state should record text only, got {kind:?}"),
            }
        }
        let content_top = commands[0].frame.y;
        let content_bottom = commands[1].frame.y + commands[1].frame.height;
        assert!((content_top - list.y) > metrics.gap_l);
        assert!((list.y + list.height - content_bottom) > metrics.gap_l);
        assert!(((content_top - list.y) - (list.y + list.height - content_bottom)).abs() <= 1.0);
    }
}
