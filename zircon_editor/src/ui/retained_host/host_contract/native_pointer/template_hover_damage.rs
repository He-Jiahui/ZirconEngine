use super::super::data::{FrameRect, HostPaneInteractionStateData};

pub(super) fn template_hover_damage(
    before: &HostPaneInteractionStateData,
    after: &HostPaneInteractionStateData,
) -> Option<FrameRect> {
    union_optional_frames(template_hover_frame(before), template_hover_frame(after))
}

fn template_hover_frame(state: &HostPaneInteractionStateData) -> Option<FrameRect> {
    (!state.hovered_template_control_id.is_empty()).then(|| state.hovered_template_frame.clone())
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
