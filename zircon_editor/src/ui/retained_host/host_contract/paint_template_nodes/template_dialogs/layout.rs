use super::super::super::data::FrameRect;
use super::identity::DialogKind;
use super::metrics::dialog_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn pixel_aligned_rect(
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x.round(),
        y: rect.y.round(),
        width: rect.width.round(),
        height: rect.height.round(),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dialog_has_visible_area(
    rect: &FrameRect,
) -> bool {
    rect.x.is_finite()
        && rect.y.is_finite()
        && rect.width.is_finite()
        && rect.height.is_finite()
        && rect.width > 0.0
        && rect.height > 0.0
        && (rect.x + rect.width).is_finite()
        && (rect.y + rect.height).is_finite()
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn frame_is_within(
    outer: &FrameRect,
    inner: &FrameRect,
) -> bool {
    dialog_has_visible_area(outer)
        && dialog_has_visible_area(inner)
        && inner.x >= outer.x
        && inner.y >= outer.y
        && inner.x + inner.width <= outer.x + outer.width
        && inner.y + inner.height <= outer.y + outer.height
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn title_rect(
    rect: &FrameRect,
) -> Option<FrameRect> {
    let metrics = dialog_metrics();
    let frame = FrameRect {
        x: content_left(rect, metrics.padding_x),
        y: rect.y + metrics.title_top,
        width: content_width(rect, metrics.padding_x),
        height: metrics.title_line_height,
    };
    frame_is_within(rect, &frame).then_some(frame)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn body_rect(
    rect: &FrameRect,
    kind: DialogKind,
    action_top: Option<f32>,
) -> Option<FrameRect> {
    let metrics = dialog_metrics();
    if matches!(kind, DialogKind::AlertDialog) {
        let frame = FrameRect {
            x: content_left(rect, metrics.padding_x),
            y: rect.y + metrics.body_top,
            width: content_width(rect, metrics.padding_x),
            height: metrics.body_line_height,
        };
        return frame_is_within(rect, &frame).then_some(frame);
    }

    let top = action_rail_floor(rect);
    let available_bottom = action_top
        .map(|action_top| {
            let preferred = action_top - metrics.content_action_gap;
            let compact = action_top - metrics.content_gap;
            if preferred - top >= metrics.body_line_height {
                preferred
            } else {
                compact
            }
        })
        .unwrap_or_else(|| rect.y + rect.height - metrics.action_bottom);
    let height = available_bottom - top;
    (height >= metrics.body_line_height)
        .then(|| FrameRect {
            x: content_left(rect, metrics.padding_x),
            y: top,
            width: content_width(rect, metrics.padding_x),
            height,
        })
        .filter(|frame| frame_is_within(rect, frame))
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
        width: metrics.severity_mark_width.min(rect.width),
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
    (rect.width - padding_x * 2.0).max(0.0)
}
