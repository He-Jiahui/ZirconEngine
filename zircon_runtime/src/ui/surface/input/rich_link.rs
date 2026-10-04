use zircon_runtime_interface::ui::{
    dispatch::{
        UiDispatchDisposition, UiDispatchEffect, UiDispatchPhase, UiInputDiagnosticsMode,
        UiInputDispatchResult,
    },
    surface::{
        UiPointerActivationPhase, UiPointerButton, UiPointerEventKind, UiPointerRoute,
        UiPointerRoutingPath,
    },
};

use crate::ui::text::link_at_layout_point;

use super::super::surface::UiSurface;
use super::effect::append_dispatch_effect_to_result;

pub(super) fn dispatch_pointer_rich_link_activation(
    surface: &mut UiSurface,
    click_count: u8,
    route: &UiPointerRoute,
    diagnostics_mode: UiInputDiagnosticsMode,
    result: &mut UiInputDispatchResult,
) {
    if !matches!(route.kind, UiPointerEventKind::Up)
        || !matches!(route.button, Some(UiPointerButton::Primary))
        || !matches!(
            route.activation_phase,
            UiPointerActivationPhase::PrimaryRelease
        )
        || !route.release_inside_pressed
        || matches!(result.reply.disposition, UiDispatchDisposition::Blocked)
    {
        return;
    }
    let Some(target) = route.click_target else {
        return;
    };
    let candidate_commands = surface
        .render_cache
        .commands_for_node(&surface.render_extract, target)
        .map(|(_, commands)| commands)
        .unwrap_or_else(|| surface.render_extract.list.commands.as_slice());
    let Some(hit) = candidate_commands
        .iter()
        .rev()
        .filter(|command| {
            command.node_id == target
                && command.frame.contains_point(route.point)
                && command
                    .clip_frame
                    .map(|clip| clip.contains_point(route.point))
                    .unwrap_or(true)
        })
        .find_map(|command| link_at_layout_point(command.text_layout.as_ref()?, route.point))
    else {
        return;
    };
    let source_range = hit.source_range;
    let affinity = hit.affinity;

    let applied_before = result.applied_effects.len();
    append_dispatch_effect_to_result(
        surface,
        result,
        UiDispatchEffect::RequestLinkActivation {
            target,
            link_target: hit.target,
        },
    );
    if result.applied_effects.len() == applied_before {
        return;
    }
    result.reply.disposition = UiDispatchDisposition::Handled;
    result.reply.handler = Some(target);
    result.reply.phase = Some(UiDispatchPhase::DefaultAction);
    result.diagnostics.routed = true;
    result.diagnostics.route_target = Some(target);
    if diagnostics_mode.captures_full_trace() {
        result.diagnostics.handled_phase = Some("pointer.rich_link_activation".to_string());
        result.diagnostics.notes.push(format!(
            "rich_link_range={}..{}:{:?}:click_count={}",
            source_range.start, source_range.end, affinity, click_count
        ));
    }
}

#[cfg(test)]
#[path = "tests/rich_link.rs"]
mod tests;
