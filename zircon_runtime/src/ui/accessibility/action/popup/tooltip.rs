use zircon_runtime_interface::ui::{
    accessibility::{UiA11yRole, UiAccessibilityActionStatus, UiAccessibilityNode},
    dispatch::{UiDispatchEffect, UiDispatchReply, UiInputDispatchResult, UiTooltipEffectKind},
    event_ui::UiNodeId,
};

use crate::ui::surface::UiSurface;

use super::super::result::action_note;

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
    result.diagnostics.routed = true;
    result.diagnostics.route_target = Some(target);
    result.diagnostics.handled_phase = Some("accessibility.dismiss_tooltip".to_string());
    result.diagnostics.notes.push(action_note(
        UiAccessibilityActionStatus::Accepted,
        None,
        None,
    ));
    result
        .diagnostics
        .notes
        .push(format!("accessibility_tooltip_hidden:{tooltip_id}"));
    result
}
