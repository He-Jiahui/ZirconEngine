use super::super::data::{FrameRect, HostPaneInteractionStateData};

pub(in crate::ui::retained_host::host_contract) fn template_hover_damage(
    before: &HostPaneInteractionStateData,
    after: &HostPaneInteractionStateData,
) -> Option<FrameRect> {
    union_optional_frames(template_hover_frame(before), template_hover_frame(after))
}

pub(in crate::ui::retained_host::host_contract) fn browser_reference_hover_damage(
    before: &HostPaneInteractionStateData,
    after: &HostPaneInteractionStateData,
) -> Option<FrameRect> {
    let before_hovered = browser_reference_hovered(before);
    let after_hovered = browser_reference_hovered(after);
    if before.browser_asset_references_hovered_index == after.browser_asset_references_hovered_index
        && before.browser_asset_used_by_hovered_index == after.browser_asset_used_by_hovered_index
    {
        return None;
    }
    match (before_hovered, after_hovered) {
        (true, true) => Some(union_frame(
            &before.browser_asset_reference_hover_frame,
            &after.browser_asset_reference_hover_frame,
        )),
        (true, false) => Some(before.browser_asset_reference_hover_frame.clone()),
        (false, true) => Some(after.browser_asset_reference_hover_frame.clone()),
        (false, false) => None,
    }
}

pub(in crate::ui::retained_host::host_contract) fn activity_reference_hover_damage(
    before: &HostPaneInteractionStateData,
    after: &HostPaneInteractionStateData,
) -> Option<FrameRect> {
    let before_hovered = activity_reference_hovered(before);
    let after_hovered = activity_reference_hovered(after);
    if before.activity_asset_references_hovered_index
        == after.activity_asset_references_hovered_index
        && before.activity_asset_used_by_hovered_index == after.activity_asset_used_by_hovered_index
    {
        return None;
    }
    match (before_hovered, after_hovered) {
        (true, true) => Some(union_frame(
            &before.activity_asset_reference_hover_frame,
            &after.activity_asset_reference_hover_frame,
        )),
        (true, false) => Some(before.activity_asset_reference_hover_frame.clone()),
        (false, true) => Some(after.activity_asset_reference_hover_frame.clone()),
        (false, false) => None,
    }
}

fn template_hover_frame(state: &HostPaneInteractionStateData) -> Option<FrameRect> {
    (!state.hovered_template_control_id.is_empty()).then(|| state.hovered_template_frame.clone())
}

fn browser_reference_hovered(state: &HostPaneInteractionStateData) -> bool {
    state.browser_asset_references_hovered_index >= 0
        || state.browser_asset_used_by_hovered_index >= 0
}

fn activity_reference_hovered(state: &HostPaneInteractionStateData) -> bool {
    state.activity_asset_references_hovered_index >= 0
        || state.activity_asset_used_by_hovered_index >= 0
}

fn union_optional_frames(left: Option<FrameRect>, right: Option<FrameRect>) -> Option<FrameRect> {
    match (left, right) {
        (Some(left), Some(right)) => Some(union_frame(&left, &right)),
        (Some(frame), None) | (None, Some(frame)) => Some(frame),
        (None, None) => None,
    }
}

fn union_frame(left: &FrameRect, right: &FrameRect) -> FrameRect {
    let x0 = left.x.min(right.x);
    let y0 = left.y.min(right.y);
    let x1 = (left.x + left.width).max(right.x + right.width);
    let y1 = (left.y + left.height).max(right.y + right.height);
    FrameRect {
        x: x0,
        y: y0,
        width: (x1 - x0).max(0.0),
        height: (y1 - y0).max(0.0),
    }
}
