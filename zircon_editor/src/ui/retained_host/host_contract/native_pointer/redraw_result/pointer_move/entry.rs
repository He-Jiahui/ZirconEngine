use crate::ui::retained_host::host_contract::data::HostPaneInteractionStateData;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;

use super::super::super::routing::{PanePointerRoute, PanePointerTarget};
use super::super::super::template_hover_damage::template_hover_damage;
use super::hierarchy::hierarchy_pointer_move_redraw;
use super::template::template_pointer_move_redraw;

pub(in crate::ui::retained_host::host_contract) fn pointer_move_redraw(
    pointer: &PanePointerRoute,
    before: &HostPaneInteractionStateData,
    after: &HostPaneInteractionStateData,
) -> NativePointerDispatchResult {
    if matches!(&pointer.target, PanePointerTarget::Viewport(_)) || before == after {
        if before == after {
            return NativePointerDispatchResult::idle();
        }
        return template_hover_damage(before, after)
            .map(NativePointerDispatchResult::region)
            .unwrap_or_else(NativePointerDispatchResult::idle);
    }

    let template_damage = template_hover_damage(before, after);
    if matches!(&pointer.target, PanePointerTarget::Hierarchy) {
        return hierarchy_pointer_move_redraw(pointer, before, after, template_damage);
    }

    if let Some(template_damage) = template_damage {
        return template_pointer_move_redraw(pointer, &template_damage);
    }

    NativePointerDispatchResult::region(pointer.frame.clone())
}
