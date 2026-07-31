use zircon_runtime_interface::ui::{
    dispatch::{UiComponentEmissionPolicy, UiComponentEventReport, UiDispatchEffect},
    event_ui::UiNodeId,
};

use super::super::super::surface::UiSurface;
use super::super::{UiSurfaceInputEffectError, UiSurfaceInputEffectResult};
use super::node::require_node;

pub(super) fn apply_component_event_effect(
    surface: &UiSurface,
    effect: &UiDispatchEffect,
) -> UiSurfaceInputEffectResult<Option<UiNodeId>> {
    match effect {
        UiDispatchEffect::EmitComponentEvent { target, policy, .. } => {
            require_node(surface, *target)?;
            match policy {
                UiComponentEmissionPolicy::Immediate
                | UiComponentEmissionPolicy::Queue
                | UiComponentEmissionPolicy::Coalesce => Ok(Some(*target)),
            }
        }
        _ => Err(UiSurfaceInputEffectError::UnexpectedEffect {
            expected: "component event",
        }),
    }
}

pub(super) fn component_event_report_for_effect(
    effect: &UiDispatchEffect,
) -> Option<UiComponentEventReport> {
    let UiDispatchEffect::EmitComponentEvent { target, event, .. } = effect else {
        return None;
    };
    Some(UiComponentEventReport {
        target: *target,
        event: event.clone(),
        delivered: true,
        drag: None,
        template_action: None,
    })
}
