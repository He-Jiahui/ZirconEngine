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
#[path = "tests/surface.rs"]
mod tests;
