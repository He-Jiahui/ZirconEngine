use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

use super::super::floating::floating_group_frame;
use super::super::groups::local_group_frame;
use super::super::union::{union_visible_frame, visible_frame};

pub(in super::super) fn release_same_group_damage_frame(
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
