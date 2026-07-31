use super::super::super::super::super::data::{FrameRect, PaneData};
use super::super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::super::paint_text::draw_text_with_size_and_style;
use super::super::super::super::super::paint_theme::current_host_metrics;
use super::super::super::super::first_non_empty;
use super::super::super::style::{WELCOME_MUTED_TEXT, WELCOME_TEXT};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const MIN_HEADER_TEXT_WIDTH: f32 = 1.0;

pub(in crate::ui::retained_host::host_contract) fn draw_welcome_new_project_header(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    header: &FrameRect,
    clip: &FrameRect,
) {
    let metrics = current_host_metrics();
    let title_font_size = metrics.font_body;
    let title_line_height = metrics
        .line_height(title_font_size)
        .round()
        .max(title_font_size.ceil());
    let template_font_size = metrics.font_small;
    let template_line_height = metrics
        .line_height(template_font_size)
        .round()
        .max(template_font_size.ceil());
    let content_height = title_line_height + metrics.gap_s + template_line_height;
    let content_y = header.y + ((header.height - content_height).max(0.0) * 0.5);
    let text_width = header.width.max(MIN_HEADER_TEXT_WIDTH);

    draw_text_with_size_and_style(
        frame,
        FrameRect {
            x: header.x,
            y: content_y,
            width: text_width,
            height: title_line_height,
        },
        "New Project",
        Some(clip),
        WELCOME_TEXT,
        title_font_size,
        title_line_height,
        UiTextRunPaintStyle::default(),
    );
    draw_text_with_size_and_style(
        frame,
        FrameRect {
            x: header.x,
            y: content_y + title_line_height + metrics.gap_s,
            width: text_width,
            height: template_line_height,
        },
        first_non_empty(&[
            pane.welcome.form.template_label.as_str(),
            "Renderable Empty",
        ]),
        Some(clip),
        WELCOME_MUTED_TEXT,
        template_font_size,
        template_line_height,
        UiTextRunPaintStyle::default(),
    );
}

#[cfg(test)]
mod tests {
    use super::super::super::super::super::super::paint_frame::HostRecordedPaintKind;
    use super::*;

    #[test]
    fn welcome_form_header_uses_shared_two_line_typography_and_relative_centering() {
        let mut frame = HostRgbaFrame::recording_only(360, 64);
        let header = FrameRect {
            x: 16.0,
            y: 12.0,
            width: 328.0,
            height: 36.0,
        };
        let clip = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 360.0,
            height: 64.0,
        };

        draw_welcome_new_project_header(&mut frame, &PaneData::default(), &header, &clip);

        let commands = frame.into_recorded_commands();
        assert_eq!(commands.len(), 2);
        let metrics = current_host_metrics();
        for (command, expected_font_size) in
            commands.iter().zip([metrics.font_body, metrics.font_small])
        {
            assert_eq!(command.frame.x, header.x);
            assert_eq!(command.frame.width, header.width);
            assert!(matches!(
                &command.kind,
                HostRecordedPaintKind::Text { font_size, .. }
                    if *font_size == expected_font_size
            ));
        }
        assert!(matches!(
            &commands[1].kind,
            HostRecordedPaintKind::Text { text, .. } if text == "Renderable Empty"
        ));
        let top_gap = commands[0].frame.y - header.y;
        let bottom_gap =
            header.y + header.height - (commands[1].frame.y + commands[1].frame.height);
        assert!((top_gap - bottom_gap).abs() <= 1.0);
    }
}
