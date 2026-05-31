use zircon_runtime_interface::ui::{
    accessibility::{UiA11yRole, UiAccessibilityNode},
    dispatch::{UiDispatchEffect, UiDispatchReply, UiInputDispatchResult, UiTooltipEffectKind},
    event_ui::UiNodeId,
};

use crate::ui::surface::UiSurface;

use super::result::finish_tooltip_dismiss;

pub(super) fn tooltip_dismissal_target(
    surface: &UiSurface,
    snapshot_node: &UiAccessibilityNode,
) -> Option<String> {
    if snapshot_node.role != UiA11yRole::Tooltip {
        return None;
    }
    surface
        .input
        .tooltip
        .as_ref()
        .map(|tooltip| tooltip.tooltip_id.clone())
}

pub(super) fn dispatch_tooltip_dismiss(
    surface: &mut UiSurface,
    target: UiNodeId,
    tooltip_id: String,
    result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    let event = result.event.clone();
    let mut notes = result.diagnostics.notes;
    let reply =
        UiDispatchReply::handled()
            .from_handler(target)
            .with_effect(UiDispatchEffect::Tooltip {
                kind: UiTooltipEffectKind::Hide,
                tooltip_id: tooltip_id.clone(),
                owner: Some(target),
            });
    let mut result = surface.apply_dispatch_reply(event, reply);
    notes.extend(result.diagnostics.notes);
    result.diagnostics.notes = notes;
    finish_tooltip_dismiss(result, target, tooltip_id)
}
