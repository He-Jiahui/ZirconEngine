use super::super::super::super::super::data::{FrameRect, PaneData};
use super::super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::super::paint_primitives::{
    draw_rect_clipped, draw_rounded_border_clipped, draw_rounded_rect_clipped,
    draw_text_bars_clipped,
};
use super::super::super::super::super::paint_text::measure_runtime_text_width;
use super::super::super::super::super::paint_theme::{
    HostControlMetrics, current_host_metrics, current_host_palette,
};
use super::super::super::super::{MUTED_TEXT, first_non_empty};

pub(in crate::ui::retained_host::host_contract::paint_workbench_renderer::docks::pane) fn draw_pane_fallback(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
) {
    let metrics = current_host_metrics();
    let pane_label = first_non_empty(&[
        pane.title.as_str(),
        pane.kind.as_str(),
        pane.subtitle.as_str(),
        pane.info.as_str(),
    ]);
    draw_text_bars_clipped(
        frame,
        body.x + metrics.gap_l,
        body.y + metrics.gap_l,
        pane_label,
        Some(clip),
        MUTED_TEXT,
    );

    let Some(card) = empty_state_card_frame(body) else {
        return;
    };
    let palette = current_host_palette();
    draw_rounded_rect_clipped(
        frame,
        card.clone(),
        Some(clip),
        palette.surface,
        metrics.radius_control,
    );
    draw_rounded_border_clipped(
        frame,
        card.clone(),
        Some(clip),
        palette.separator_soft,
        metrics.border_width,
        metrics.radius_control,
    );
    draw_empty_state_marker(frame, &card, clip, palette.separator_strong, &metrics);

    let label = fallback_state_label(pane);
    let label_x = centered_text_x(&card, label, metrics.font_body);
    draw_text_bars_clipped(
        frame,
        label_x,
        card.y + card.height * 0.62,
        label,
        Some(&card),
        palette.text,
    );
}

pub(super) fn empty_state_card_frame(body: &FrameRect) -> Option<FrameRect> {
    if !body.width.is_finite()
        || !body.height.is_finite()
        || body.width <= 0.0
        || body.height <= 0.0
    {
        return None;
    }

    let metrics = current_host_metrics();
    let horizontal_inset = metrics.gap_l * 2.0;
    let vertical_inset = metrics.gap_l * 2.0;
    let minimum_width = metrics.row_height * 4.0;
    let minimum_height = metrics.row_height * 3.0;
    let available_width = body.width - horizontal_inset;
    let available_height = body.height - vertical_inset;
    if available_width < minimum_width || available_height < minimum_height {
        return None;
    }

    let width = (body.width * 0.62)
        .max(minimum_width)
        .min(metrics.row_height * 13.0)
        .min(available_width);
    let height = (body.height * 0.24)
        .max(minimum_height)
        .min(metrics.row_height * 5.0)
        .min(available_height);
    Some(FrameRect {
        x: body.x + (body.width - width) * 0.5,
        y: body.y + (body.height - height) * 0.5,
        width,
        height,
    })
}

fn draw_empty_state_marker(
    frame: &mut HostRgbaFrame,
    card: &FrameRect,
    clip: &FrameRect,
    color: [u8; 4],
    metrics: &HostControlMetrics,
) {
    let marker_height = metrics.border_width.max(1.0);
    let marker_gap = metrics.gap_s.max(metrics.border_width);
    let marker_width = (card.width * 0.16)
        .min(metrics.row_height * 2.0)
        .max(metrics.row_height);
    let total_height = marker_height * 3.0 + marker_gap * 2.0;
    let x = card.x + (card.width - marker_width) * 0.5;
    let y = card.y + (card.height * 0.34 - total_height * 0.5).max(metrics.gap_m);
    for index in 0..3 {
        let scale = 1.0 - index as f32 * 0.18;
        let width = marker_width * scale;
        draw_rect_clipped(
            frame,
            FrameRect {
                x: x + (marker_width - width) * 0.5,
                y: y + index as f32 * (marker_height + marker_gap),
                width,
                height: marker_height,
            },
            Some(clip),
            color,
        );
    }
}

fn centered_text_x(card: &FrameRect, text: &str, font_size: f32) -> f32 {
    let text_width = measure_runtime_text_width(text, font_size);
    card.x + ((card.width - text_width) * 0.5).max(0.0)
}

fn fallback_state_label(pane: &PaneData) -> &str {
    match pane.kind.as_str() {
        "Scene" => "No scene content",
        "Game" => "No game content",
        "Inspector" => "No selection",
        "Hierarchy" => "No actors",
        "Assets" | "AssetBrowser" => "No assets",
        _ => first_non_empty(&[
            pane.subtitle.as_str(),
            pane.info.as_str(),
            pane.title.as_str(),
            pane.kind.as_str(),
        ]),
    }
}
