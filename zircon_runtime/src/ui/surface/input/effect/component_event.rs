use zircon_runtime_interface::ui::{
    dispatch::{UiComponentEmissionPolicy, UiComponentEventReport, UiDispatchEffect},
    event_ui::UiNodeId,
};

use super::super::super::surface::UiSurface;
use super::node::require_node;

pub(super) fn apply_component_event_effect(
    surface: &UiSurface,
    effect: &UiDispatchEffect,
) -> Result<Option<UiNodeId>, String> {
    match effect {
        UiDispatchEffect::EmitComponentEvent { target, policy, .. } => {
            require_node(surface, *target)?;
            match policy {
                UiComponentEmissionPolicy::Immediate
                | UiComponentEmissionPolicy::Queue
                | UiComponentEmissionPolicy::Coalesce => Ok(Some(*target)),
            }
        }
        _ => Err("expected component event effect".to_string()),
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
    })
}
