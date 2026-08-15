use super::super::super::super::super::data::{FrameRect, PaneData};
use super::super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::super::paint_primitives::{
    draw_rounded_border_clipped, draw_rounded_rect_clipped,
};
use super::super::super::super::super::paint_text::draw_text_with_size_and_style;
use super::super::super::super::super::paint_theme::current_host_metrics;
use super::super::super::super::{first_non_empty, SEPARATOR};
use super::super::super::style::{WELCOME_MUTED_TEXT, WELCOME_SURFACE, WELCOME_TEXT};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const MIN_PREVIEW_TEXT_WIDTH: f32 = 1.0;

pub(in crate::ui::retained_host::host_contract) fn draw_welcome_preview(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    preview: &FrameRect,
    clip: &FrameRect,
) {
    let metrics = current_host_metrics();
    draw_rounded_rect_clipped(
        frame,
        preview.clone(),
        Some(clip),
        WELCOME_SURFACE,
        metrics.radius_control,
    );
    draw_rounded_border_clipped(
        frame,
        preview.clone(),
        Some(clip),
        SEPARATOR,
        metrics.border_width,
        metrics.radius_control,
    );

    let label_font_size = metrics.font_small;
    let label_line_height = metrics
        .line_height(label_font_size)
        .round()
        .max(label_font_size.ceil());
    let path_font_size = metrics.font_body;
    let path_line_height = metrics
        .line_height(path_font_size)
        .round()
        .max(path_font_size.ceil());
    let content_height = label_line_height + metrics.gap_s + path_line_height;
    let content_y = preview.y + ((preview.height - content_height).max(0.0) * 0.5);
    let text_x = preview.x + metrics.gap_l;
    let text_width = (preview.width - metrics.gap_l * 2.0).max(MIN_PREVIEW_TEXT_WIDTH);

    draw_text_with_size_and_style(
        frame,
        FrameRect {
            x: text_x,
            y: content_y,
            width: text_width,
            height: label_line_height,
        },
        "Project path",
        Some(clip),
        WELCOME_MUTED_TEXT,
        label_font_size,
        label_line_height,
        UiTextRunPaintStyle::default(),
    );
    draw_text_with_size_and_style(
        frame,
        FrameRect {
            x: text_x,
            y: content_y + label_line_height + metrics.gap_s,
            width: text_width,
            height: path_line_height,
        },
        first_non_empty(&[
            pane.welcome.form.project_path_preview.as_str(),
            "Project path will appear here",
        ]),
        Some(clip),
        if pane.welcome.form.project_path_preview.is_empty() {
            WELCOME_MUTED_TEXT
        } else {
            WELCOME_TEXT
        },
        path_font_size,
        path_line_height,
        UiTextRunPaintStyle::default(),
    );
}

#[cfg(test)]
mod tests {
    use super::super::super::super::super::super::paint_frame::HostRecordedPaintKind;
    use super::*;

    #[test]
    fn welcome_path_preview_uses_shared_surface_and_runtime_text_inset() {
        let mut frame = HostRgbaFrame::recording_only(360, 96);
        let mut pane = PaneData::default();
        pane.welcome.form.project_path_preview = "E:/Projects/ZirconProject".into();
        let preview = FrameRect {
            x: 16.0,
            y: 12.0,
            width: 328.0,
            height: 72.0,
        };
        let clip = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 360.0,
            height: 96.0,
        };

        draw_welcome_preview(&mut frame, &pane, &preview, &clip);

        let commands = frame.into_recorded_commands();
        assert_eq!(commands.len(), 4);
        let metrics = current_host_metrics();
        assert!(matches!(
            &commands[0].kind,
            HostRecordedPaintKind::Quad { corner_radius, .. }
                if *corner_radius == metrics.radius_control
        ));
        assert!(matches!(
            &commands[1].kind,
            HostRecordedPaintKind::Border { width, corner_radius, .. }
                if *width == metrics.border_width && *corner_radius == metrics.radius_control
        ));
        for (command, expected_font_size) in commands[2..]
            .iter()
            .zip([metrics.font_small, metrics.font_body])
        {
            assert_eq!(command.frame.x, preview.x + metrics.gap_l);
            assert_eq!(command.frame.width, preview.width - metrics.gap_l * 2.0);
            assert!(matches!(
                &command.kind,
                HostRecordedPaintKind::Text { font_size, .. }
                    if *font_size == expected_font_size
            ));
        }
        assert!(matches!(
            &commands[3].kind,
            HostRecordedPaintKind::Text { text, color, .. }
                if text == "E:/Projects/ZirconProject" && *color == WELCOME_TEXT
        ));
    }
}
