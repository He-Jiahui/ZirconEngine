use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

use super::super::floating::{floating_group_frame, floating_target_damage_frame};
use super::super::groups::{document_edge_group, local_group_frame};
use super::super::regions::{
    center_band_status_damage_frame, center_band_status_with_source_damage_frame,
};

pub(super) fn cross_group_tab_drag_release_damage_frame(
    presentation: &HostWindowPresentationData,
    source_group: &str,
    target_group: &str,
) -> Option<FrameRect> {
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
