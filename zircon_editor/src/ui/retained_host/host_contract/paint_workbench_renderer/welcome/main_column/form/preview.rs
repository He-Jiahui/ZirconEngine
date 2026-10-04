use super::super::super::super::super::data::{FrameRect, PaneData};
use super::super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::super::paint_primitives::draw_rounded_box_clipped;
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
    draw_rounded_box_clipped(
        frame,
        preview.clone(),
        Some(clip),
        WELCOME_SURFACE,
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
#[path = "tests/preview.rs"]
mod tests;
