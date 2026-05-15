use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

pub(super) fn close_prompt_action_damage_frame(
    presentation: &HostWindowPresentationData,
) -> Option<FrameRect> {
    let prompt = &presentation.close_prompt;
    if !prompt.visible {
        return None;
    }
    union_visible_frame(
        visible_frame(&prompt.overlay_frame).then_some(prompt.overlay_frame.clone()),
        prompt.dialog_frame.clone(),
    )
}

fn union_visible_frame(current: Option<FrameRect>, frame: FrameRect) -> Option<FrameRect> {
    if !visible_frame(&frame) {
        return current;
    }
    Some(match current {
        Some(current) => union_frame(&current, &frame),
        None => frame,
    })
}

fn visible_frame(frame: &FrameRect) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width > 0.0
        && frame.height > 0.0
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
