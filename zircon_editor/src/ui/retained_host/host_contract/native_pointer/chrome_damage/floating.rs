use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

use super::union::{union_visible_frame, visible_frame};

pub(super) fn floating_document_tab_damage_frame(
    presentation: &HostWindowPresentationData,
    surface_key: &str,
) -> Option<FrameRect> {
    let scene = &presentation.host_scene_data;
    let window = (0..scene.floating_layer.floating_windows.row_count()).find_map(|row| {
        let window = scene.floating_layer.floating_windows.row_data(row)?;
        (window.window_id.as_str() == surface_key).then_some(window)
    })?;
    let frame = window.frame.clone();
    visible_frame(&frame).then_some(frame)
}

pub(super) fn floating_window_header_damage_frame(
    presentation: &HostWindowPresentationData,
    window_id: &str,
) -> Option<FrameRect> {
    let scene = &presentation.host_scene_data;
    let target = (0..scene.floating_layer.floating_windows.row_count()).find_map(|row| {
        let window = scene.floating_layer.floating_windows.row_data(row)?;
        (window.window_id.as_str() == window_id).then_some(window.frame)
    })?;
    let mut damage = Some(target);
    for row in 0..scene.floating_layer.floating_windows.row_count() {
        let Some(window) = scene.floating_layer.floating_windows.row_data(row) else {
            continue;
        };
        damage = union_visible_frame(damage, window.frame.clone());
    }
    damage
}
