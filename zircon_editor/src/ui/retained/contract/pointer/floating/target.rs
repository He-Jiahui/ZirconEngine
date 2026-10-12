use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

use super::super::groups::{document_edge_group, local_group_frame};
use super::super::regions::{center_band_status_damage_frame, release_same_group_damage_frame};
use super::super::union::union_visible_frame;

pub(in super::super) fn floating_target_damage_frame(
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
