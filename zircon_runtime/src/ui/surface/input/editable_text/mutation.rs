use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    component::{UiComponentEvent, UiValue},
    dispatch::{UiComponentEventReport, UiDispatchReply, UiInputDispatchResult, UiInputEvent},
    event_ui::{UiNodeId, UiReflectedPropertySource},
    surface::UiEditableTextState,
};

use crate::ui::surface::{UiPropertyMutationRequest, UiPropertyMutationStatus};

use super::super::super::surface::UiSurface;
use super::super::{
    effect::append_dispatch_effect_to_result,
    owner_route::{focused_input_kind_for_event, record_owner_focused_input},
    text_state::editable_value_property,
};
use super::ime_context::input_method_update_for_text_state;

pub(in crate::ui::surface::input) fn apply_editable_text_state(
    surface: &mut UiSurface,
    event: UiInputEvent,
    target: UiNodeId,
    state: UiEditableTextState,
    phase: &str,
    component_event_kind: TextComponentEventKind,
) -> UiInputDispatchResult {
    let kind = focused_input_kind_for_event(&event);
    let mut result = UiInputDispatchResult::new(event, UiDispatchReply::handled());
    result.diagnostics.routed = true;
    result.diagnostics.route_target = Some(target);
    result.diagnostics.handled_phase = Some(phase.to_string());
    if let Some(kind) = kind {
        record_owner_focused_input(surface, kind, target, Some(target), true);
    }

    let Some(value_property) = editable_value_property(surface, target) else {
        result
            .diagnostics
            .notes
            .push("editable value property missing".to_string());
        return result;
    };

    let value_changed = mutate_text_property(
        surface,
        target,
        value_property.as_str(),
        UiValue::String(state.text.clone()),
        &mut result,
    );
    mutate_text_property(
        surface,
        target,
        "caret_offset",
        UiValue::Int(state.caret.offset as i64),
        &mut result,
    );
    let (selection_anchor, selection_focus) = state
        .selection
        .as_ref()
        .map(|selection| (selection.anchor, selection.focus))
        .unwrap_or((state.caret.offset, state.caret.offset));
    mutate_text_property(
        surface,
        target,
        "selection_anchor",
        UiValue::Int(selection_anchor as i64),
        &mut result,
    );
    mutate_text_property(
        surface,
        target,
        "selection_focus",
        UiValue::Int(selection_focus as i64),
        &mut result,
    );

    let (composition_start, composition_end, composition_text, restore_text) = state
        .composition
        .as_ref()
        .map(|composition| {
            (
                composition.range.start,
                composition.range.end,
                composition.text.clone(),
                composition.restore_text.clone().unwrap_or_default(),
            )
        })
        .unwrap_or((
            state.caret.offset,
            state.caret.offset,
            String::new(),
            String::new(),
        ));
    mutate_text_property(
        surface,
        target,
        "composition_start",
        UiValue::Int(composition_start as i64),
        &mut result,
    );
    mutate_text_property(
        surface,
        target,
        "composition_end",
        UiValue::Int(composition_end as i64),
        &mut result,
    );
    mutate_text_property(
        surface,
        target,
        "composition_text",
        UiValue::String(composition_text),
        &mut result,
    );
    mutate_text_property(
        surface,
        target,
        "composition_restore_text",
        UiValue::String(restore_text),
        &mut result,
    );
    push_text_component_event_report(
        surface,
        target,
        value_property.as_str(),
        &state,
        component_event_kind,
        value_changed,
        &mut result,
    );
    if let Some(effect) = input_method_update_for_text_state(surface, &result.event, target, &state)
    {
        append_dispatch_effect_to_result(surface, &mut result, effect);
    }

    result
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::ui::surface::input) enum TextComponentEventKind {
    Change,
    Submit,
}

fn push_text_component_event_report(
    surface: &UiSurface,
    target: UiNodeId,
    value_property: &str,
    state: &UiEditableTextState,
    kind: TextComponentEventKind,
    value_changed: bool,
    result: &mut UiInputDispatchResult,
) {
    if kind == TextComponentEventKind::Change && !value_changed {
        return;
    }
    let Some(metadata) = surface
        .tree
        .nodes
        .get(&target)
        .and_then(|node| node.template_metadata.as_ref())
    else {
        return;
    };
    let binding_event = match kind {
        TextComponentEventKind::Change => UiEventKind::Change,
        TextComponentEventKind::Submit => UiEventKind::Submit,
    };
    if !metadata
        .bindings
        .iter()
        .any(|binding| binding.event == binding_event)
    {
        return;
    }
    let event = match kind {
        TextComponentEventKind::Change => UiComponentEvent::ValueChanged {
            property: value_property.to_string(),
            value: UiValue::String(state.text.clone()),
        },
        TextComponentEventKind::Submit => UiComponentEvent::Commit {
            property: value_property.to_string(),
            value: UiValue::String(state.text.clone()),
        },
    };
    result.component_events.push(UiComponentEventReport {
        target,
        event,
        delivered: true,
        drag: result.drag,
        template_action: None,
    });
}

fn mutate_text_property(
    surface: &mut UiSurface,
    target: UiNodeId,
    property: &str,
    value: UiValue,
    result: &mut UiInputDispatchResult,
) -> bool {
    let report = surface.mutate_property(
        UiPropertyMutationRequest::widget_behavior(target, property, value)
            .with_source(UiReflectedPropertySource::RuntimeState),
    );
    match report {
        Ok(report) => {
            result.record_binding_report(report.binding.clone());
            if matches!(report.status, UiPropertyMutationStatus::Accepted) {
                result.diagnostics.notes.push(format!(
                    "text_property_changed:{}:{:?}",
                    report.property, report.invalidation.dirty
                ));
                return true;
            }
        }
        Err(error) => result
            .diagnostics
            .notes
            .push(format!("text_property_rejected:{property}:{error}")),
    }
    false
}
