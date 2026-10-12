use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

use super::super::floating::floating_group_frame;
use super::super::union::union_visible_frame;

pub(in super::super) fn center_band_status_damage_frame(
    presentation: &HostWindowPresentationData,
) -> Option<FrameRect> {
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

pub(in super::super) fn center_band_status_with_source_damage_frame(
    presentation: &HostWindowPresentationData,
    source_group: &str,
) -> Option<FrameRect> {
    let mut damage = center_band_status_damage_frame(presentation);
    if let Some(source_frame) = floating_group_frame(presentation, source_group) {
        damage = union_visible_frame(damage, source_frame);
    }
    damage
}
