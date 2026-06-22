use crate::ui::retained_host::host_contract::data::HostPaneInteractionStateData;
use crate::ui::retained_host::host_contract::frame_geometry::union_frame;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::surface_hit_test::TemplateNodePointerHit;

use super::super::template_hover_damage::template_hover_damage;

pub(in crate::ui::retained_host::host_contract) fn workbench_template_node_move_redraw(
    hit: &TemplateNodePointerHit,
    before: &HostPaneInteractionStateData,
    after: &HostPaneInteractionStateData,
) -> NativePointerDispatchResult {
    if before == after {
        return NativePointerDispatchResult::idle();
    }
    template_hover_damage(before, after)
        .map(|template| union_frame(&template, &hit.frame))
        .map(NativePointerDispatchResult::region)
        .unwrap_or_else(|| NativePointerDispatchResult::region(hit.frame.clone()))
}
