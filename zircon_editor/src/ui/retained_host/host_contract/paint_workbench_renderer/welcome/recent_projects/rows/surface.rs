use super::super::super::super::super::data::FrameRect;
use super::super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::super::paint_primitives::{
    draw_rect_clipped, draw_rounded_border_clipped, draw_rounded_box_clipped,
};
use super::super::super::super::super::paint_text::{
    draw_text_with_size_and_style, measure_runtime_text_width,
};
use super::super::super::super::super::paint_theme::{current_host_metrics, HostControlMetrics};
use super::super::super::super::SEPARATOR;
use super::super::super::style::{
    WELCOME_MUTED_TEXT, WELCOME_SURFACE, WELCOME_SURFACE_INSET, WELCOME_TEXT, WELCOME_WARNING,
};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const MIN_ACTION_LABEL_SIZE: f32 = 1.0;

pub(super) fn draw_recent_project_row_surface(
    frame: &mut HostRgbaFrame,
    row: &FrameRect,
    clip: &FrameRect,
    invalid: bool,
) {
    let metrics = current_host_metrics();
    draw_rect_clipped(frame, row.clone(), Some(clip), WELCOME_SURFACE);
    let separator_height = metrics.border_width.min(row.height.max(0.0));
    draw_rect_clipped(
        frame,
        FrameRect {
            x: row.x,
            y: (row.y + row.height - separator_height).max(row.y),
            width: row.width,
            height: separator_height,
        },
        Some(clip),
        SEPARATOR,
    );
    if invalid {
        draw_rounded_border_clipped(
            frame,
            row.clone(),
            Some(clip),
            WELCOME_WARNING,
            metrics.border_width,
            metrics.radius_control,
        );
    }
}

pub(super) fn draw_recent_project_row_actions(
    frame: &mut HostRgbaFrame,
    open: &FrameRect,
    safe: &FrameRect,
    recover: &FrameRect,
    remove: &FrameRect,
    clip: &FrameRect,
    invalid: bool,
) {
    let metrics = current_host_metrics();
    for action in [open, safe, recover, remove] {
        draw_rounded_box_clipped(
            frame,
            action.clone(),
            Some(clip),
            WELCOME_SURFACE_INSET,
            if invalid { WELCOME_WARNING } else { SEPARATOR },
            metrics.border_width,
            metrics.radius_control,
        );
    }
    draw_recent_project_action_label(
        frame,
        open,
        clip,
        "Open",
        if invalid {
            WELCOME_MUTED_TEXT
        } else {
            WELCOME_TEXT
        },
        metrics,
    );
    draw_recent_project_action_label(frame, safe, clip, "S", WELCOME_WARNING, metrics);
    draw_recent_project_action_label(
        frame,
        recover,
        clip,
        "R",
        if invalid {
            WELCOME_MUTED_TEXT
        } else {
            WELCOME_TEXT
        },
        metrics,
    );
    draw_recent_project_action_label(frame, remove, clip, "×", WELCOME_MUTED_TEXT, metrics);
}

fn draw_recent_project_action_label(
    frame: &mut HostRgbaFrame,
    action: &FrameRect,
    clip: &FrameRect,
    label: &str,
    color: [u8; 4],
    metrics: HostControlMetrics,
) {
    let font_size = metrics
        .font_body
        .min(action.height.max(MIN_ACTION_LABEL_SIZE));
    let line_height = metrics
        .line_height(font_size)
        .round()
        .max(font_size.ceil())
        .min(action.height.max(MIN_ACTION_LABEL_SIZE));
    let label_width = (measure_runtime_text_width(label, font_size) + metrics.text_clip_guard)
        .min(action.width.max(MIN_ACTION_LABEL_SIZE))
        .max(MIN_ACTION_LABEL_SIZE);
    let label_frame = FrameRect {
        x: action.x + ((action.width - label_width).max(0.0) * 0.5),
        y: action.y + ((action.height - line_height).max(0.0) * 0.5),
        width: label_width,
        height: line_height,
    };
    draw_text_with_size_and_style(
        frame,
        label_frame,
        label,
        Some(clip),
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
    fn recent_project_rows_keep_idle_surfaces_flat_and_reserve_outline_for_warning_state() {
        let row = FrameRect {
            x: 8.0,
            y: 8.0,
            width: 104.0,
            height: 32.0,
        };
        let clip = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 128.0,
            height: 48.0,
        };

        let mut idle_frame = HostRgbaFrame::recording_only(128, 48);
        draw_recent_project_row_surface(&mut idle_frame, &row, &clip, false);
        let idle_commands = idle_frame.into_recorded_commands();
        assert_eq!(idle_commands.len(), 2);
        assert!(idle_commands
            .iter()
            .all(|command| matches!(&command.kind, HostRecordedPaintKind::Quad { .. })));
        match &idle_commands[1].kind {
            HostRecordedPaintKind::Quad {
                color,
                corner_radius,
            } => {
                assert_eq!(*color, SEPARATOR);
                assert_eq!(*corner_radius, 0.0);
            }
            ref kind => panic!("idle row separator should be a flat quad, got {kind:?}"),
        }
        let metrics = current_host_metrics();
        assert_eq!(idle_commands[1].frame.height, metrics.border_width);

        let mut warning_frame = HostRgbaFrame::recording_only(128, 48);
        draw_recent_project_row_surface(&mut warning_frame, &row, &clip, true);
        let warning_commands = warning_frame.into_recorded_commands();
        assert_eq!(warning_commands.len(), 3);
        match &warning_commands[2].kind {
            HostRecordedPaintKind::Border {
                color,
                width,
                corner_radius,
            } => {
                assert_eq!(*color, WELCOME_WARNING);
                assert_eq!(*width, metrics.border_width);
                assert_eq!(*corner_radius, metrics.radius_control);
            }
            ref kind => panic!("warning row should add one semantic outline, got {kind:?}"),
        }
    }

    #[test]
    fn recent_project_actions_use_shared_control_radius_for_fill_and_border() {
        let mut frame = HostRgbaFrame::recording_only(160, 48);
        let open = FrameRect {
            x: 8.0,
            y: 8.0,
            width: 52.0,
            height: 24.0,
        };
        let recover = FrameRect {
            x: 100.0,
            y: 8.0,
            width: 24.0,
            height: 24.0,
        };
        let remove = FrameRect {
            x: 132.0,
            y: 8.0,
            width: 24.0,
            height: 24.0,
        };
        let clip = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 160.0,
            height: 48.0,
        };

        let safe = FrameRect {
            x: 68.0,
            y: 8.0,
            width: 24.0,
            height: 24.0,
        };

        draw_recent_project_row_actions(&mut frame, &open, &safe, &recover, &remove, &clip, false);

        let metrics = current_host_metrics();
        let commands = frame.into_recorded_commands();
        let surface_commands = commands
            .iter()
            .filter(|command| {
                matches!(
                    &command.kind,
                    HostRecordedPaintKind::Quad { .. } | HostRecordedPaintKind::Border { .. }
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(surface_commands.len(), 8);
        for command in surface_commands {
            match &command.kind {
                HostRecordedPaintKind::Quad { corner_radius, .. } => {
                    assert_eq!(*corner_radius, metrics.radius_control);
                }
                HostRecordedPaintKind::Border {
                    width,
                    corner_radius,
                    ..
                } => {
                    assert_eq!(*width, metrics.border_width);
                    assert_eq!(*corner_radius, metrics.radius_control);
                }
                _ => unreachable!("surface filter only retains quads and borders"),
            }
        }
        let text_commands = commands
            .iter()
            .filter(|command| matches!(&command.kind, HostRecordedPaintKind::Text { .. }))
            .collect::<Vec<_>>();
        assert_eq!(text_commands.len(), 4);
        for (command, action) in text_commands.iter().zip([&open, &safe, &recover, &remove]) {
            match &command.kind {
                HostRecordedPaintKind::Text {
                    font_size,
                    line_height,
                    ..
                } => {
                    assert_eq!(*font_size, metrics.font_body);
                    assert_eq!(*line_height, metrics.line_height(metrics.font_body).round());
                }
                _ => unreachable!("text filter only retains text commands"),
            }
            let label_center_x = command.frame.x + command.frame.width * 0.5;
            let label_center_y = command.frame.y + command.frame.height * 0.5;
            assert!((label_center_x - (action.x + action.width * 0.5)).abs() <= 1.0);
            assert!((label_center_y - (action.y + action.height * 0.5)).abs() <= 1.0);
        }
        assert!(matches!(
            &text_commands[1].kind,
            HostRecordedPaintKind::Text { text, .. } if text == "S"
        ));
        assert!(matches!(
            &text_commands[2].kind,
            HostRecordedPaintKind::Text { text, .. } if text == "R"
        ));
        assert!(matches!(
            &text_commands[3].kind,
            HostRecordedPaintKind::Text { text, .. } if text == "×"
        ));
    }
}
