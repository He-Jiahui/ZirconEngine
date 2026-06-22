mod target;

use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostDragStateData, HostWindowPresentationData,
};

use self::target::cross_group_tab_drag_release_damage_frame;

pub(in crate::ui::retained_host::host_contract) fn tab_drag_release_damage_frame(
    presentation: &HostWindowPresentationData,
    drag_state: &HostDragStateData,
) -> Option<FrameRect> {
    let source_group = drag_state.drag_source_group.as_str();
    let target_group = drag_state.active_drag_target_group.as_str();
    if source_group == target_group {
        return super::regions::release_same_group_damage_frame(presentation, target_group);
    }
    cross_group_tab_drag_release_damage_frame(presentation, source_group, target_group)
}
