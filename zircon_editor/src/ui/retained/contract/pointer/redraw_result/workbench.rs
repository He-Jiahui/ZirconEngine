use crate::ui::retained_host::host_contract::data::HostPaneInteractionStateData;
use crate::ui::retained_host::host_contract::frame_geometry::union_frame;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;

use super::super::template_hover_damage::template_hover_damage;

pub(in crate::ui::retained_host::host_contract) fn workbench_template_node_move_redraw(
    hit_frame: &crate::ui::retained_host::host_contract::data::FrameRect,
    before: &HostPaneInteractionStateData,
    after: &HostPaneInteractionStateData,
) -> NativePointerDispatchResult {
    if before == after {
        return NativePointerDispatchResult::idle();
    }
    template_hover_damage(before, after)
        .map(|template| union_frame(&template, hit_frame))
        .map(NativePointerDispatchResult::region)
        .unwrap_or_else(|| NativePointerDispatchResult::region(hit_frame.clone()))
}
