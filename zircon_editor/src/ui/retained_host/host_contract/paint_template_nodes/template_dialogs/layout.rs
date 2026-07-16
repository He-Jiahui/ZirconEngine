use super::super::super::data::FrameRect;
use super::identity::DialogKind;
use super::metrics::dialog_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn pixel_aligned_rect(
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x.round(),
        y: rect.y.round(),
        width: rect.width.round().max(1.0),
        height: rect.height.round().max(1.0),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dialog_has_visible_area(
    rect: &FrameRect,
) -> bool {
    rect.width > 1.0 && rect.height > 1.0
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn title_rect(
    rect: &FrameRect,
) -> FrameRect {
    let metrics = dialog_metrics();
    FrameRect {
        x: content_left(rect, metrics.padding_x),
        y: rect.y + metrics.title_top,
        width: content_width(rect, metrics.padding_x),
        height: metrics.title_line_height,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn body_rect(
    rect: &FrameRect,
    kind: DialogKind,
    action_top: Option<f32>,
) -> Option<FrameRect> {
    let metrics = dialog_metrics();
    if matches!(kind, DialogKind::AlertDialog) {
        return Some(FrameRect {
            x: content_left(rect, metrics.padding_x),
            y: rect.y + metrics.body_top,
            width: content_width(rect, metrics.padding_x),
            height: metrics.body_line_height,
        });
    }

    let top = action_rail_floor(rect);
    let body_bottom = top + metrics.body_line_height;
    let body_fits = action_top
        .map(|action_top| {
            let available_gap = action_top - body_bottom;
            available_gap.min(metrics.content_action_gap) >= metrics.content_gap
        })
        .unwrap_or_else(|| body_bottom <= rect.y + rect.height - metrics.action_bottom);
    body_fits.then(|| FrameRect {
        x: content_left(rect, metrics.padding_x),
        y: top,
        width: content_width(rect, metrics.padding_x),
        height: metrics.body_line_height,
    })
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn action_rail_floor(
    rect: &FrameRect,
) -> f32 {
    let metrics = dialog_metrics();
    rect.y + metrics.title_top + metrics.title_line_height + metrics.content_gap
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn severity_mark_rect(
    rect: &FrameRect,
) -> FrameRect {
    let metrics = dialog_metrics();
    FrameRect {
        x: rect.x,
        y: rect.y,
        width: metrics.severity_mark_width,
        height: rect.height,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn action_right(
    rect: &FrameRect,
) -> f32 {
    rect.x + rect.width - dialog_metrics().padding_x
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn action_available_width(
    rect: &FrameRect,
) -> f32 {
    content_width(rect, dialog_metrics().padding_x)
}

fn content_left(rect: &FrameRect, padding_x: f32) -> f32 {
    rect.x + padding_x
}

fn content_width(rect: &FrameRect, padding_x: f32) -> f32 {
    (rect.width - padding_x * 2.0).max(1.0)
}
