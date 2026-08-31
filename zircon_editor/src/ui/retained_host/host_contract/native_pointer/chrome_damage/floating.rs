use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

use super::union::{union_visible_frame, visible_frame};

pub(super) fn floating_document_tab_damage_frame(
    presentation: &HostWindowPresentationData,
    surface_key: &str,
) -> Option<FrameRect> {
    let scene = &presentation.host_scene_data;
    let window = scene
        .floating_layer
        .floating_windows
        .iter()
        .find(|window| window.window_id.as_str() == surface_key)?;
    let frame = window.frame.clone();
    visible_frame(&frame).then_some(frame)
}

pub(super) fn floating_window_header_damage_frame(
    presentation: &HostWindowPresentationData,
    window_id: &str,
) -> Option<FrameRect> {
    let scene = &presentation.host_scene_data;
    let target = scene
        .floating_layer
        .floating_windows
        .iter()
        .find(|window| window.window_id.as_str() == window_id)
        .map(|window| window.frame.clone())?;
    let mut damage = Some(target);
    for window in scene.floating_layer.floating_windows.iter() {
        damage = union_visible_frame(damage, window.frame.clone());
    }
    damage
}
