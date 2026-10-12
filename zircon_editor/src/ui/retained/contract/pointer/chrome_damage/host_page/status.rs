use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

use super::super::union::union_visible_frame;

pub(super) fn union_host_page_status_damage(
    mut damage: Option<FrameRect>,
    presentation: &HostWindowPresentationData,
) -> Option<FrameRect> {
    let scene = &presentation.host_scene_data;
    damage = union_visible_frame(damage, presentation.host_layout.center_band_frame.clone());
    damage = union_visible_frame(damage, scene.layout.center_band_frame.clone());
    damage = union_visible_frame(damage, presentation.host_layout.status_bar_frame.clone());
    damage = union_visible_frame(damage, scene.layout.status_bar_frame.clone());
    union_visible_frame(damage, scene.status_bar.status_bar_frame.clone())
}
