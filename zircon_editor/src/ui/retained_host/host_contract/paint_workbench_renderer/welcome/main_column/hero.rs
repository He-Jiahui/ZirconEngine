use super::super::super::super::data::{FrameRect, PaneData};
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_primitives::{
    draw_rect_clipped, draw_rounded_border_clipped, draw_rounded_rect_clipped,
};
use super::super::super::super::paint_text::{
    draw_text_with_size_and_style, measure_runtime_text_width,
};
use super::super::super::super::paint_theme::current_host_metrics;
use super::super::super::{ACCENT, SEPARATOR, first_non_empty};
use super::super::style::{
    WELCOME_MUTED_TEXT, WELCOME_SUCCESS, WELCOME_SURFACE_INSET, WELCOME_TEXT,
};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const MIN_WELCOME_TEXT_SLOT_WIDTH: f32 = 1.0;

pub(in crate::ui::retained_host::host_contract) fn draw_welcome_hero(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    hero: &FrameRect,
    clip: &FrameRect,
) {
    let metrics = current_host_metrics();
    let title = first_non_empty(&[pane.welcome.title.as_str(), "Open or Create"]);
    let title_font_size = metrics.font_large;
    let title_line_height = shared_line_height(title_font_size);
    let subtitle_font_size = metrics.font_body;
    let subtitle_line_height = shared_line_height(subtitle_font_size);
    let accent_height = metrics.selection_indicator_width.min(hero.height.max(0.0));
    let content_height = title_line_height + metrics.gap_s + subtitle_line_height;
    let text_height = (hero.height - accent_height).max(0.0);
    let content_y = hero.y + ((text_height - content_height).max(0.0) * 0.5);
    let text_width = hero.width.max(MIN_WELCOME_TEXT_SLOT_WIDTH);

    draw_text_with_size_and_style(
        frame,
        FrameRect {
            x: hero.x,
            y: content_y,
            width: text_width,
            height: title_line_height,
        },
        title,
        Some(clip),
        WELCOME_TEXT,
        title_font_size,
        title_line_height,
        UiTextRunPaintStyle::default(),
    );
    draw_text_with_size_and_style(
        frame,
        FrameRect {
            x: hero.x,
            y: content_y + title_line_height + metrics.gap_s,
            width: text_width,
            height: subtitle_line_height,
        },
        first_non_empty(&[
            pane.welcome.subtitle.as_str(),
            "Recent projects and a renderable empty-project template",
        ]),
        Some(clip),
        WELCOME_MUTED_TEXT,
        subtitle_font_size,
        subtitle_line_height,
        UiTextRunPaintStyle::default(),
    );

    let accent_y = hero.y + hero.height - accent_height;
    draw_rect_clipped(
        frame,
        FrameRect {
            x: hero.x,
            y: accent_y,
            width: hero.width,
            height: metrics.border_width.min(accent_height),
        },
        Some(clip),
        SEPARATOR,
    );
    draw_rect_clipped(
        frame,
        FrameRect {
            x: hero.x,
            y: accent_y,
            width: (measure_runtime_text_width(title, title_font_size) + metrics.text_clip_guard)
                .min(hero.width.max(0.0)),
            height: accent_height,
        },
        Some(clip),
        ACCENT,
    );
}

pub(in crate::ui::retained_host::host_contract) fn draw_welcome_status(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    status: &FrameRect,
    clip: &FrameRect,
) {
    let metrics = current_host_metrics();
    draw_rounded_rect_clipped(
        frame,
        status.clone(),
        Some(clip),
        WELCOME_SURFACE_INSET,
        metrics.radius_control,
    );
    draw_rounded_border_clipped(
        frame,
        status.clone(),
        Some(clip),
        SEPARATOR,
        metrics.border_width,
        metrics.radius_control,
    );

    let marker_size = metrics.gap_m.min(status.height.max(0.0));
    let marker = FrameRect {
        x: status.x + metrics.gap_l,
        y: status.y + ((status.height - marker_size).max(0.0) * 0.5),
        width: marker_size,
        height: marker_size,
    };
    draw_rounded_rect_clipped(
        frame,
        marker.clone(),
        Some(clip),
        WELCOME_SUCCESS,
        marker_size * 0.5,
    );

    let font_size = metrics.font_body;
    let line_height = shared_line_height(font_size).min(status.height.max(0.0));
    let text_x = marker.x + marker.width + metrics.gap_m;
    draw_text_with_size_and_style(
        frame,
        FrameRect {
            x: text_x,
            y: status.y + ((status.height - line_height).max(0.0) * 0.5),
            width: (status.x + status.width - metrics.gap_l - text_x)
                .max(MIN_WELCOME_TEXT_SLOT_WIDTH),
            height: line_height,
        },
        first_non_empty(&[pane.welcome.status_message.as_str(), "Ready"]),
        Some(clip),
        WELCOME_MUTED_TEXT,
        font_size,
        line_height,
        UiTextRunPaintStyle::default(),
    );
}

fn shared_line_height(font_size: f32) -> f32 {
    let metrics = current_host_metrics();
    metrics.line_height(font_size).round().max(font_size.ceil())
}

#[cfg(test)]
mod tests {
    use super::super::super::super::super::paint_frame::HostRecordedPaintKind;
    use super::*;

    fn test_clip() -> FrameRect {
        FrameRect {
            x: 0.0,
            y: 0.0,
            width: 360.0,
            height: 160.0,
        }
    }

    #[test]
    fn welcome_hero_uses_runtime_text_metrics_and_measured_accent_width() {
        let mut frame = HostRgbaFrame::recording_only(360, 160);
        let mut pane = PaneData::default();
        pane.welcome.title = "Create a project".into();
        pane.welcome.subtitle = "Start from a renderable template".into();
        let hero = FrameRect {
            x: 16.0,
            y: 12.0,
            width: 328.0,
            height: 84.0,
        };

        draw_welcome_hero(&mut frame, &pane, &hero, &test_clip());

        let commands = frame.into_recorded_commands();
        assert_eq!(commands.len(), 4);
        let metrics = current_host_metrics();
        assert!(matches!(
            &commands[0].kind,
            HostRecordedPaintKind::Text { text, font_size, .. }
                if text == "Create a project" && *font_size == metrics.font_large
        ));
        assert!(matches!(
            &commands[1].kind,
            HostRecordedPaintKind::Text { text, font_size, .. }
                if text == "Start from a renderable template" && *font_size == metrics.font_body
        ));
        assert_eq!(commands[0].frame.x, hero.x);
        assert_eq!(commands[0].frame.width, hero.width);
        assert_eq!(commands[3].frame.height, metrics.selection_indicator_width);
        assert!(commands[3].frame.width < hero.width);
        assert!(commands[3].frame.width > metrics.gap_l);
    }

    #[test]
    fn welcome_status_uses_shared_radius_border_and_vertically_centered_runtime_text() {
        let mut frame = HostRgbaFrame::recording_only(360, 64);
        let status = FrameRect {
            x: 16.0,
            y: 12.0,
            width: 328.0,
            height: 30.0,
        };

        draw_welcome_status(&mut frame, &PaneData::default(), &status, &test_clip());

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
        assert!(matches!(
            &commands[3].kind,
            HostRecordedPaintKind::Text { text, font_size, .. }
                if text == "Ready" && *font_size == metrics.font_body
        ));
        let text_center_y = commands[3].frame.y + commands[3].frame.height * 0.5;
        assert!((text_center_y - (status.y + status.height * 0.5)).abs() <= 1.0);
    }
}
