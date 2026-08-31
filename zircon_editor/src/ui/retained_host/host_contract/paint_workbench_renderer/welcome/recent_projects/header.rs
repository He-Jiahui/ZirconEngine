use super::super::super::super::data::{FrameRect, WelcomePaneLayoutData};
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_text::draw_text_with_size_and_style;
use super::super::super::super::paint_theme::current_host_metrics;
use super::super::layout::translated_welcome_frame;
use super::super::style::WELCOME_TEXT;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const RECENT_HEADER_TOP_INSET: f32 = 18.0;
const RECENT_HEADER_HEIGHT: f32 = 30.0;

pub(super) fn recent_projects_header_frame(
    layout: &WelcomePaneLayoutData,
    body: &FrameRect,
    recent_panel: &FrameRect,
) -> FrameRect {
    translated_welcome_frame(layout.recent_header_panel.as_ref(), body).unwrap_or_else(|| {
        FrameRect {
            x: recent_panel.x,
            y: recent_panel.y + RECENT_HEADER_TOP_INSET,
            width: recent_panel.width,
            height: RECENT_HEADER_HEIGHT,
        }
    })
}

pub(super) fn draw_recent_projects_header(
    frame: &mut HostRgbaFrame,
    header: &FrameRect,
    clip: &FrameRect,
) {
    let metrics = current_host_metrics();
    let title_font_size = metrics.font_body;
    let title_line_height = metrics
        .line_height(title_font_size)
        .round()
        .max(title_font_size.ceil());
    let content_y = header.y + ((header.height - title_line_height).max(0.0) * 0.5);
    let text_x = header.x + metrics.gap_l;
    let text_width = (header.width - metrics.gap_l * 2.0).max(1.0);

    draw_text_with_size_and_style(
        frame,
        FrameRect {
            x: text_x,
            y: content_y,
            width: text_width,
            height: title_line_height,
        },
        "Recent Projects",
        Some(clip),
        WELCOME_TEXT,
        title_font_size,
        title_line_height,
        UiTextRunPaintStyle::default(),
    );
}

#[cfg(test)]
mod tests {
    use super::super::super::super::super::paint_frame::HostRecordedPaintKind;
    use super::*;

    #[test]
    fn recent_header_uses_shared_typography_and_centers_single_line_content() {
        let mut frame = HostRgbaFrame::recording_only(320, 64);
        let header = FrameRect {
            x: 8.0,
            y: 4.0,
            width: 304.0,
            height: RECENT_HEADER_HEIGHT,
        };
        let clip = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 320.0,
            height: 64.0,
        };

        draw_recent_projects_header(&mut frame, &header, &clip);

        let commands = frame.into_recorded_commands();
        assert_eq!(commands.len(), 1);
        let metrics = current_host_metrics();
        for command in &commands {
            assert_eq!(command.frame.x, header.x + metrics.gap_l);
            assert_eq!(command.frame.width, header.width - metrics.gap_l * 2.0);
            match &command.kind {
                HostRecordedPaintKind::Text { font_size, .. } => {
                    assert_eq!(*font_size, metrics.font_body);
                }
                kind => panic!("recent header should record text only, got {kind:?}"),
            }
        }
        assert!(commands[0].frame.y >= header.y);
        assert!(commands[0].frame.y + commands[0].frame.height <= header.y + header.height);
    }
}
