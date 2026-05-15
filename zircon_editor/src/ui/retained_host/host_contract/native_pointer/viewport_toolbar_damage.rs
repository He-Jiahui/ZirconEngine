use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

pub(super) fn viewport_toolbar_press_damage_frame(
    presentation: &HostWindowPresentationData,
    control_id: &str,
    toolbar_frame: &FrameRect,
    extra_damage: Option<FrameRect>,
) -> Option<FrameRect> {
    let base_damage = if viewport_toolbar_click_affects_viewport_or_status(control_id) {
        center_band_status_damage_frame(presentation)
    } else {
        visible_frame(toolbar_frame).then_some(toolbar_frame.clone())
    };
    union_optional_frames(base_damage, extra_damage)
}

fn viewport_toolbar_click_affects_viewport_or_status(control_id: &str) -> bool {
    matches!(
        control_id,
        "EnterPlayMode" | "ExitPlayMode" | "frame.selection" | "frame_selection"
    ) || control_id.starts_with("align.")
}

fn center_band_status_damage_frame(presentation: &HostWindowPresentationData) -> Option<FrameRect> {
    let mut damage = None;
    let scene = &presentation.host_scene_data;

    // View alignment, frame selection, and play-mode toggles can update viewport
    // body and status text. They should not repaint menu/title chrome.
    damage = union_visible_frame(damage, presentation.host_layout.center_band_frame.clone());
    damage = union_visible_frame(damage, scene.layout.center_band_frame.clone());
    damage = union_visible_frame(damage, presentation.host_layout.status_bar_frame.clone());
    damage = union_visible_frame(damage, scene.layout.status_bar_frame.clone());
    damage = union_visible_frame(damage, scene.status_bar.status_bar_frame.clone());
    damage
}

fn union_optional_frames(left: Option<FrameRect>, right: Option<FrameRect>) -> Option<FrameRect> {
    match (left, right) {
        (Some(left), Some(right)) => Some(union_frame(&left, &right)),
        (Some(frame), None) | (None, Some(frame)) => Some(frame),
        (None, None) => None,
    }
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
