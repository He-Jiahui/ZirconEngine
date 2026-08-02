use super::super::super::super::super::data::FrameRect;
use super::super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::super::paint_geometry::intersect;
use super::super::super::super::super::paint_text::{
    draw_text_with_size_and_style, measure_runtime_text_width,
};
use super::super::super::super::super::paint_theme::{HostControlMetrics, current_host_metrics};
use super::super::super::style::{WELCOME_MUTED_TEXT, WELCOME_TEXT, WELCOME_WARNING};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const STATUS_MAX_WIDTH_FRACTION: f32 = 0.42;
const MIN_TEXT_SLOT_WIDTH: f32 = 1.0;

pub(super) fn draw_recent_project_row_text(
    frame: &mut HostRgbaFrame,
    row: &FrameRect,
    text: &FrameRect,
    clip: &FrameRect,
    display_name: &str,
    path: &str,
    status: &str,
    invalid: bool,
) {
    if intersect(text, clip).is_none() {
        return;
    }
    let metrics = current_host_metrics();
    let font_size = metrics.font_body;
    let line_height = metrics.line_height(font_size).round().max(font_size.ceil());
    let title_y = row.y + metrics.gap_m;
    let status_width = recent_status_width(text.width, status, font_size, metrics);
    let status_gap = if status_width > 0.0 {
        metrics.gap_m
    } else {
        0.0
    };
    let title_width = (text.width - status_width - status_gap).max(MIN_TEXT_SLOT_WIDTH);

    draw_recent_text_slot(
        frame,
        FrameRect {
            x: text.x,
            y: title_y,
            width: title_width,
            height: line_height,
        },
        clip,
        display_name,
        WELCOME_TEXT,
        font_size,
        line_height,
    );
    draw_recent_text_slot(
        frame,
        FrameRect {
            x: text.x,
            y: title_y + line_height + metrics.gap_s,
            width: text.width.max(MIN_TEXT_SLOT_WIDTH),
            height: line_height,
        },
        clip,
        path,
        WELCOME_MUTED_TEXT,
        font_size,
        line_height,
    );
    if status_width <= 0.0 {
        return;
    }
    draw_recent_text_slot(
        frame,
        FrameRect {
            x: text.x + text.width - status_width,
            y: title_y,
            width: status_width,
            height: line_height,
        },
        clip,
        status,
        if invalid {
            WELCOME_WARNING
        } else {
            WELCOME_MUTED_TEXT
        },
        font_size,
        line_height,
    );
}

fn recent_status_width(
    available_width: f32,
    status: &str,
    font_size: f32,
    metrics: HostControlMetrics,
) -> f32 {
    if status.is_empty() || available_width <= 0.0 {
        return 0.0;
    }
    let measured_width = measure_runtime_text_width(status, font_size) + metrics.text_clip_guard;
    let fractional_cap = (available_width * STATUS_MAX_WIDTH_FRACTION).max(MIN_TEXT_SLOT_WIDTH);
    let title_reserving_cap =
        (available_width - metrics.gap_m - MIN_TEXT_SLOT_WIDTH).max(MIN_TEXT_SLOT_WIDTH);
    measured_width
        .min(fractional_cap)
        .min(title_reserving_cap)
        .max(MIN_TEXT_SLOT_WIDTH)
}

fn draw_recent_text_slot(
    frame: &mut HostRgbaFrame,
    slot: FrameRect,
    clip: &FrameRect,
    text: &str,
    color: [u8; 4],
    font_size: f32,
    line_height: f32,
) {
    let Some(slot_clip) = intersect(&slot, clip) else {
        return;
    };
    draw_text_with_size_and_style(
        frame,
        slot,
        text,
        Some(&slot_clip),
        color,
        font_size,
        line_height,
        UiTextRunPaintStyle::default(),
    );
}

#[cfg(test)]
mod tests {
    use super::super::super::super::super::super::paint_frame::HostRecordedPaintKind;
    use super::*;

    #[test]
    fn recent_project_title_reserves_measured_status_slot_and_uses_shared_text_metrics() {
        let mut frame = HostRgbaFrame::recording_only(320, 64);
        let row = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 300.0,
            height: 54.0,
        };
        let text = FrameRect {
            x: 8.0,
            y: 0.0,
            width: 220.0,
            height: 54.0,
        };
        let clip = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 320.0,
            height: 64.0,
        };

        draw_recent_project_row_text(
            &mut frame,
            &row,
            &text,
            &clip,
            "A very long project name that must not overlap status",
            "E:/Projects/Zircon",
            "Missing",
            true,
        );

        let commands = frame
            .into_recorded_commands()
            .into_iter()
            .filter(|command| matches!(&command.kind, HostRecordedPaintKind::Text { .. }))
            .collect::<Vec<_>>();
        assert_eq!(commands.len(), 3);
        let metrics = current_host_metrics();
        let expected_line_height = metrics
            .line_height(metrics.font_body)
            .round()
            .max(metrics.font_body.ceil());
        for command in &commands {
            match &command.kind {
                HostRecordedPaintKind::Text {
                    font_size,
                    line_height,
                    ..
                } => {
                    assert_eq!(*font_size, metrics.font_body);
                    assert_eq!(*line_height, expected_line_height);
                }
                _ => unreachable!("text filter only retains text commands"),
            }
        }
        assert!(commands[0].frame.x + commands[0].frame.width <= commands[2].frame.x);
        assert!(commands[1].frame.width > commands[0].frame.width);
        assert!(matches!(
            &commands[2].kind,
            HostRecordedPaintKind::Text { text, color, .. }
                if text == "Missing" && *color == WELCOME_WARNING
        ));
    }
}
