use crate::ui::template_runtime::RetainedUiHostBindingProjection;
use zircon_runtime_interface::ui::binding::UiEventKind;

use super::path::binding_path_action_id;

pub(in super::super) fn primary_click_binding_id(
    bindings: &[RetainedUiHostBindingProjection],
) -> Option<String> {
    bindings
        .iter()
        .find(|binding| binding.event_kind == UiEventKind::Click)
        .map(|binding| binding.binding_id.clone())
}

pub(in super::super) fn primary_click_action_id(
    bindings: &[RetainedUiHostBindingProjection],
) -> Option<String> {
    bindings
        .iter()
        .find(|binding| binding.event_kind == UiEventKind::Click)
        .and_then(|binding| (!binding.action_id.is_empty()).then(|| binding.action_id.clone()))
}

pub(in super::super) fn primary_change_action_id(
    bindings: &[RetainedUiHostBindingProjection],
) -> Option<String> {
    bindings
        .iter()
        .find(|binding| binding.event_kind == UiEventKind::Change)
        .map(|binding| binding_path_action_id(&binding.binding_id))
}

pub(in super::super) fn primary_submit_action_id(
    bindings: &[RetainedUiHostBindingProjection],
) -> Option<String> {
    bindings
        .iter()
        .find(|binding| binding.event_kind == UiEventKind::Submit)
        .map(|binding| binding_path_action_id(&binding.binding_id))
}
