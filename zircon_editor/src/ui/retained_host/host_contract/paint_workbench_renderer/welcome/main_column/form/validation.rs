use super::super::super::super::super::data::{FrameRect, PaneData};
use super::super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::super::paint_primitives::draw_rounded_rect_clipped;
use super::super::super::super::super::paint_text::draw_text_with_size_and_style;
use super::super::super::super::super::paint_theme::current_host_metrics;
use super::super::super::style::{WELCOME_SUCCESS, WELCOME_WARNING};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const MIN_VALIDATION_TEXT_WIDTH: f32 = 1.0;

pub(in crate::ui::retained_host::host_contract) fn draw_welcome_validation(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    validation: &FrameRect,
    clip: &FrameRect,
) {
    let message = if !pane.welcome.form.validation_message.trim().is_empty() {
        pane.welcome.form.validation_message.as_str()
    } else if pane.welcome.form.can_create {
        "Project settings are valid"
    } else {
        "Enter a project name and location"
    };
    let color = if pane.welcome.form.can_create {
        WELCOME_SUCCESS
    } else {
        WELCOME_WARNING
    };
    let metrics = current_host_metrics();
    let marker_size = metrics.gap_m.min(validation.height.max(0.0));
    let marker = FrameRect {
        x: validation.x,
        y: validation.y + ((validation.height - marker_size).max(0.0) * 0.5),
        width: marker_size,
        height: marker_size,
    };
    draw_rounded_rect_clipped(frame, marker.clone(), Some(clip), color, marker_size * 0.5);

    let font_size = metrics.font_body;
    let line_height = metrics
        .line_height(font_size)
        .round()
        .max(font_size.ceil())
        .min(validation.height.max(0.0));
    let text_x = marker.x + marker.width + metrics.gap_m;
    draw_text_with_size_and_style(
        frame,
        FrameRect {
            x: text_x,
            y: validation.y + ((validation.height - line_height).max(0.0) * 0.5),
            width: (validation.x + validation.width - text_x).max(MIN_VALIDATION_TEXT_WIDTH),
            height: line_height,
        },
        message,
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
    fn welcome_validation_uses_semantic_marker_and_centered_runtime_text_for_each_state() {
        let validation = FrameRect {
            x: 16.0,
            y: 8.0,
            width: 328.0,
            height: 32.0,
        };
        let clip = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 360.0,
            height: 48.0,
        };
        let metrics = current_host_metrics();

        for (can_create, expected_message, expected_color) in [
            (false, "Enter a project name and location", WELCOME_WARNING),
            (true, "Project settings are valid", WELCOME_SUCCESS),
        ] {
            let mut frame = HostRgbaFrame::recording_only(360, 48);
            let mut pane = PaneData::default();
            pane.welcome.form.can_create = can_create;

            draw_welcome_validation(&mut frame, &pane, &validation, &clip);

            let commands = frame.into_recorded_commands();
            assert_eq!(commands.len(), 2);
            assert!(matches!(
                &commands[0].kind,
                HostRecordedPaintKind::Quad { color, corner_radius }
                    if *color == expected_color && *corner_radius == metrics.gap_m * 0.5
            ));
            assert!(matches!(
                &commands[1].kind,
                HostRecordedPaintKind::Text { text, color, font_size, .. }
                    if text == expected_message
                        && *color == expected_color
                        && *font_size == metrics.font_body
            ));
            let text_center_y = commands[1].frame.y + commands[1].frame.height * 0.5;
            assert!((text_center_y - (validation.y + validation.height * 0.5)).abs() <= 1.0);
        }
    }
}
