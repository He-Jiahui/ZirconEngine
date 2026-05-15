use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostDragStateData, HostWindowPresentationData,
};

pub(super) fn tab_drag_release_damage_frame(
    presentation: &HostWindowPresentationData,
    drag_state: &HostDragStateData,
) -> Option<FrameRect> {
    let source_group = drag_state.drag_source_group.as_str();
    let target_group = drag_state.active_drag_target_group.as_str();
    if source_group == target_group {
        return release_same_group_damage_frame(presentation, target_group);
    }
    if local_group_frame(presentation, source_group).is_some()
        && local_group_frame(presentation, target_group).is_some()
    {
        return center_band_status_damage_frame(presentation);
    }
    if document_edge_group(target_group) {
        return center_band_status_with_source_damage_frame(presentation, source_group);
    }
    if let Some(target_frame) = floating_group_frame(presentation, target_group) {
        return floating_target_damage_frame(presentation, source_group, target_frame);
    }
    if local_group_frame(presentation, target_group).is_some()
        && floating_group_frame(presentation, source_group).is_some()
    {
        return center_band_status_with_source_damage_frame(presentation, source_group);
    }
    None
}

fn release_same_group_damage_frame(
    presentation: &HostWindowPresentationData,
    group: &str,
) -> Option<FrameRect> {
    let frame = local_group_frame(presentation, group)
        .or_else(|| floating_group_frame(presentation, group))?;
    let scene = &presentation.host_scene_data;
    let mut damage = visible_frame(&frame).then_some(frame);
    damage = union_visible_frame(damage, presentation.host_layout.status_bar_frame.clone());
    damage = union_visible_frame(damage, scene.layout.status_bar_frame.clone());
    damage = union_visible_frame(damage, scene.status_bar.status_bar_frame.clone());
    damage
}

fn center_band_status_damage_frame(presentation: &HostWindowPresentationData) -> Option<FrameRect> {
    let scene = &presentation.host_scene_data;
    let mut damage = None;

    // Cross-dock tab drops can move several panes but do not mutate menu/title chrome.
    damage = union_visible_frame(damage, presentation.host_layout.center_band_frame.clone());
    damage = union_visible_frame(damage, scene.layout.center_band_frame.clone());
    damage = union_visible_frame(damage, presentation.host_layout.status_bar_frame.clone());
    damage = union_visible_frame(damage, scene.layout.status_bar_frame.clone());
    damage = union_visible_frame(damage, scene.status_bar.status_bar_frame.clone());
    damage
}

fn center_band_status_with_source_damage_frame(
    presentation: &HostWindowPresentationData,
    source_group: &str,
) -> Option<FrameRect> {
    let mut damage = center_band_status_damage_frame(presentation);
    if let Some(source_frame) = floating_group_frame(presentation, source_group) {
        damage = union_visible_frame(damage, source_frame);
    }
    damage
}

fn floating_target_damage_frame(
    presentation: &HostWindowPresentationData,
    source_group: &str,
    target_frame: FrameRect,
) -> Option<FrameRect> {
    let mut damage = if local_group_frame(presentation, source_group).is_some()
        || document_edge_group(source_group)
    {
        center_band_status_damage_frame(presentation)
    } else {
        release_same_group_damage_frame(presentation, source_group)
    };
    damage = union_visible_frame(damage, target_frame);
    damage
}

fn local_group_frame(presentation: &HostWindowPresentationData, group: &str) -> Option<FrameRect> {
    let scene = &presentation.host_scene_data;
    let frame = match group {
        "left" => scene.left_dock.region_frame.clone(),
        "document" => scene.document_dock.region_frame.clone(),
        "right" => scene.right_dock.region_frame.clone(),
        "bottom" => scene.bottom_dock.region_frame.clone(),
        _ => return None,
    };
    visible_frame(&frame).then_some(frame)
}

fn document_edge_group(group: &str) -> bool {
    matches!(
        group,
        "document-left" | "document-right" | "document-top" | "document-bottom"
    )
}

fn floating_group_frame(
    presentation: &HostWindowPresentationData,
    group: &str,
) -> Option<FrameRect> {
    let scene = &presentation.host_scene_data;
    for row in 0..scene.floating_layer.floating_windows.row_count() {
        let Some(window) = scene.floating_layer.floating_windows.row_data(row) else {
            continue;
        };
        if group == window.target_group.as_str()
            || group == window.left_edge_target_group.as_str()
            || group == window.right_edge_target_group.as_str()
            || group == window.top_edge_target_group.as_str()
            || group == window.bottom_edge_target_group.as_str()
        {
            return visible_frame(&window.frame).then_some(window.frame.clone());
        }
    }
    None
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
