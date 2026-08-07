use super::super::super::data::{paint_menu_state, FrameRect, HostWindowPresentationData};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_primitives::draw_border_clipped;
use super::super::super::paint_text::draw_text_with_size_and_style;
use super::super::super::paint_theme::{
    current_host_metrics, current_host_palette, HostControlMetrics,
};
use super::geometry::scrolled_menu_frame;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract) fn draw_menu_bar_labels(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    let scene = &presentation.host_scene_data;
    let metrics = current_host_metrics();
    let palette = current_host_palette();
    let menu_state = paint_menu_state(presentation);
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: scene
            .layout
            .status_bar_frame
            .width
            .max(scene.layout.center_band_frame.width),
        height: scene.menu_chrome.top_bar_height_px.max(0.0),
    };
    for row in 0..scene.menu_chrome.menu_frames.row_count() {
        let Some(menu_frame) = scene.menu_chrome.menu_frames.row_data(row) else {
            continue;
        };
        let Some(menu) = scene.menu_chrome.menus.row_data(row) else {
            continue;
        };
        let color = if menu_state.open_menu_index == row as i32 {
            palette.accent
        } else {
            palette.text_muted
        };
        let frame_rect = scrolled_menu_frame(&menu_frame.frame, presentation);
        draw_menu_bar_label(
            frame,
            menu.label.as_str(),
            &frame_rect,
            Some(&clip),
            color,
            metrics,
        );
        draw_border_clipped(frame, frame_rect, Some(&clip), palette.border);
    }
}

fn draw_menu_bar_label(
    frame: &mut HostRgbaFrame,
    text: &str,
    menu_frame: &FrameRect,
    clip: Option<&FrameRect>,
    color: [u8; 4],
    metrics: HostControlMetrics,
) {
    let line_height = metrics
        .line_height(metrics.font_body)
        .round()
        .max(metrics.font_body.ceil());
    let horizontal_inset = (metrics.gap_m - metrics.border_width * 2.0).max(0.0);
    let text_frame = menu_bar_label_frame(menu_frame, horizontal_inset, line_height);
    draw_text_with_size_and_style(
        frame,
        text_frame,
        text,
        clip,
        color,
        metrics.font_body,
        line_height,
        UiTextRunPaintStyle::default(),
    );
}

fn menu_bar_label_frame(
    menu_frame: &FrameRect,
    horizontal_inset: f32,
    line_height: f32,
) -> FrameRect {
    let line_height = line_height.min(menu_frame.height.max(1.0));
    FrameRect {
        x: menu_frame.x + horizontal_inset,
        y: menu_frame.y + (menu_frame.height - line_height).max(0.0) * 0.5,
        width: (menu_frame.width - horizontal_inset * 2.0).max(1.0),
        height: line_height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_frame::HostRecordedPaintKind;
    use crate::ui::retained_host::host_contract::paint_theme::METRICS;

    #[test]
    fn menu_bar_label_frame_is_finite_and_centered() {
        let menu_frame = FrameRect {
            x: 24.0,
            y: 8.0,
            width: 96.0,
            height: 28.0,
        };

        let label = menu_bar_label_frame(&menu_frame, 6.0, 16.0);

        assert_eq!(label.x, 30.0);
        assert_eq!(label.y, 14.0);
        assert_eq!(label.width, 84.0);
        assert_eq!(label.height, 16.0);
    }

    #[test]
    fn menu_bar_label_uses_a_finite_runtime_text_slot_with_ellipsis() {
        let menu_frame = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 72.0,
            height: 28.0,
        };
        let mut frame = HostRgbaFrame::recording_only(80, 32);

        draw_menu_bar_label(
            &mut frame,
            "A long workbench menu label that must ellipsize",
            &menu_frame,
            Some(&menu_frame),
            [230, 230, 230, 255],
            METRICS,
        );

        let command = frame
            .into_recorded_commands()
            .into_iter()
            .find(|command| matches!(&command.kind, HostRecordedPaintKind::Text { .. }))
            .expect("menu bar label should use Runtime Text");
        let HostRecordedPaintKind::Text { text, .. } = &command.kind else {
            unreachable!("filtered command should be text");
        };

        assert!(text.ends_with('\u{2026}'));
        assert!(command.frame.x >= menu_frame.x);
        assert!(command.frame.x + command.frame.width <= menu_frame.x + menu_frame.width);
    }
}
