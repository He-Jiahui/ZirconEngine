use crate::ui::retained_host::host_contract::data::HostPaneInteractionStateData;
use crate::ui::retained_host::host_contract::frame_geometry::union_frame;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;

use super::super::super::routing::{PanePointerRoute, PanePointerTarget};
use super::super::super::template_hover_damage::{
    activity_reference_hover_damage, browser_reference_hover_damage, template_hover_damage,
};
use super::hierarchy::hierarchy_pointer_move_redraw;
use super::template::template_pointer_move_redraw;

pub(in crate::ui::retained_host::host_contract) fn pointer_move_redraw(
    pointer: &PanePointerRoute,
    before: &HostPaneInteractionStateData,
    after: &HostPaneInteractionStateData,
) -> NativePointerDispatchResult {
    if before == after {
        return NativePointerDispatchResult::idle();
    }

    let template_damage = template_hover_damage(before, after);
    let reference_damage = merge_hover_damage(
        browser_reference_hover_damage(before, after),
        activity_reference_hover_damage(before, after),
    );
    let damage = merge_hover_damage(template_damage, reference_damage);
    if matches!(
        &pointer.target,
        PanePointerTarget::SceneViewport(_) | PanePointerTarget::GameViewport(_)
    ) {
        return damage
            .map(NativePointerDispatchResult::region)
            .unwrap_or_else(NativePointerDispatchResult::idle);
    }
    if matches!(&pointer.target, PanePointerTarget::Hierarchy) {
        return hierarchy_pointer_move_redraw(pointer, before, after, damage);
    }

    if let Some(damage) = damage {
        return template_pointer_move_redraw(pointer, &damage);
    }

    NativePointerDispatchResult::region(pointer.frame.clone())
}

fn merge_hover_damage(
    template_damage: Option<crate::ui::retained_host::host_contract::data::FrameRect>,
    reference_damage: Option<crate::ui::retained_host::host_contract::data::FrameRect>,
) -> Option<crate::ui::retained_host::host_contract::data::FrameRect> {
    match (template_damage, reference_damage) {
        (Some(template_damage), Some(reference_damage)) => {
            Some(union_frame(&template_damage, &reference_damage))
        }
        (Some(damage), None) | (None, Some(damage)) => Some(damage),
        (None, None) => None,
    }
}

#[cfg(test)]
#[path = "tests/entry.rs"]
mod tests;
