use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

use super::union::union_visible_frame;

pub(super) fn center_band_status_damage_frame(
    presentation: &HostWindowPresentationData,
) -> Option<FrameRect> {
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
