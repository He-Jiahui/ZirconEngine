use zircon_runtime_interface::ui::{
    accessibility::{
        UiA11yRole, UiAccessibilityAction, UiAccessibilityActionStatus, UiAccessibilityNode,
        UiAccessibilityTreeSnapshot,
    },
    component::{UiComponentEvent, UiValue},
    dispatch::{
        UiAccessibilityInputEvent, UiComponentEventReport, UiDispatchReply, UiInputDispatchResult,
        UiInputEvent,
    },
    event_ui::{UiNodeId, UiReflectedPropertySource},
    focus::{UiFocusChangeReason, UiFocusVisible, UiFocusVisibleReason},
};

use crate::ui::surface::{UiPropertyMutationRequest, UiPropertyMutationStatus, UiSurface};
use crate::ui::tree::UiRuntimeTreeScrollExt;

pub(crate) fn dispatch_accessibility_action(
    surface: &mut UiSurface,
    event: UiAccessibilityInputEvent,
) -> UiInputDispatchResult {
    let request = event.request.clone();
    let mut result = UiInputDispatchResult::new(
        UiInputEvent::Accessibility(event),
        UiDispatchReply::unhandled(),
    );
    let target = request.target;
    let snapshot = surface.accessibility_snapshot();
    let Some(snapshot_node) = snapshot.node(target).cloned() else {
        return reject_missing_target(surface, &snapshot, target, result);
    };

    append_target_diagnostics(&snapshot, target, &mut result);
    if snapshot_node.state.hidden {
        return finish_unhandled(
            result,
            Some(target),
            UiAccessibilityActionStatus::Rejected,
            "hidden_target",
            "target is hidden in the accessibility snapshot",
        );
    }
    if snapshot_node.state.disabled && request.action != UiAccessibilityAction::Focus {
        return finish_unhandled(
            result,
            Some(target),
            UiAccessibilityActionStatus::Rejected,
            "disabled_action",
            "disabled accessibility target rejected non-focus action",
        );
    }

    match request.action {
        UiAccessibilityAction::Focus => dispatch_focus(surface, target, &snapshot_node, result),
        UiAccessibilityAction::Activate => {
            dispatch_activate(surface, target, &snapshot_node, result)
        }
        UiAccessibilityAction::SetValue => {
            dispatch_set_value(surface, &request, &snapshot_node, result)
        }
        UiAccessibilityAction::Increment | UiAccessibilityAction::Decrement => {
            dispatch_adjust_value(surface, &request, &snapshot_node, result)
        }
        UiAccessibilityAction::ScrollTo => {
            dispatch_scroll_to(surface, &request, &snapshot_node, result)
        }
        UiAccessibilityAction::Dismiss => finish_unhandled_with_note(
            result,
            Some(target),
            UiAccessibilityActionStatus::Unsupported,
            "unsupported_role_action",
            "accessibility dismiss requires popup id",
            "accessibility dismiss requires popup id",
        ),
    }
}

fn dispatch_scroll_to(
    surface: &mut UiSurface,
    request: &zircon_runtime_interface::ui::accessibility::UiAccessibilityActionRequest,
    snapshot_node: &UiAccessibilityNode,
    result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    let target = request.target;
    if !snapshot_node
        .actions
        .contains(&UiAccessibilityAction::ScrollTo)
    {
        return unsupported_role_action(result, target, "target does not expose scroll action");
    }
    let Some(offset) = scroll_to_offset(request) else {
        return finish_unhandled(
            result,
            Some(target),
            UiAccessibilityActionStatus::Rejected,
            "missing_value",
            "scroll to action requires value or numeric_value",
        );
    };

    match surface.tree.set_scroll_offset(target, offset as f32) {
        Ok(true) => finish_handled(result, target, "accessibility.scroll_to"),
        Ok(false) => {
            let mut result = finish_handled(result, target, "accessibility.scroll_to");
            result
                .diagnostics
                .notes
                .push("accessibility_scroll_unchanged".to_string());
            result
        }
        Err(error) => finish_unhandled(
            result,
            Some(target),
            UiAccessibilityActionStatus::Rejected,
            "mutation_error",
            &format!("scroll to action failed: {error}"),
        ),
    }
}

fn scroll_to_offset(
    request: &zircon_runtime_interface::ui::accessibility::UiAccessibilityActionRequest,
) -> Option<f64> {
    request
        .numeric_value
        .or_else(|| {
            request
                .value
                .as_deref()
                .and_then(|value| value.parse::<f64>().ok())
        })
        .filter(|value| value.is_finite())
}

fn dispatch_adjust_value(
    surface: &mut UiSurface,
    request: &zircon_runtime_interface::ui::accessibility::UiAccessibilityActionRequest,
    snapshot_node: &UiAccessibilityNode,
    result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    let target = request.target;
    if !snapshot_node.actions.contains(&request.action) {
        return unsupported_role_action(result, target, "target does not expose adjust action");
    }
    if snapshot_node.role != UiA11yRole::Slider {
        return unsupported_role_action(result, target, "adjust value requires slider role");
    }
    let direction = match request.action {
        UiAccessibilityAction::Increment => 1.0,
        UiAccessibilityAction::Decrement => -1.0,
        _ => unreachable!("dispatch_adjust_value only handles increment/decrement"),
    };
    match surface.mutate_default_range_step_value(target, direction) {
        Ok(Some((report, value)))
            if matches!(
                report.status,
                UiPropertyMutationStatus::Accepted | UiPropertyMutationStatus::Unchanged
            ) =>
        {
            let mut result = finish_handled(result, target, "accessibility.adjust_value");
            result.diagnostics.notes.push(format!(
                "accessibility_property_changed:{}:{:?}",
                report.property, report.invalidation.dirty
            ));
            if matches!(report.status, UiPropertyMutationStatus::Accepted) {
                result.component_events.push(UiComponentEventReport {
                    target,
                    event: UiComponentEvent::ValueChanged {
                        property: report.property,
                        value: UiValue::Float(value),
                    },
                    delivered: true,
                });
            }
            result
        }
        Ok(Some((report, _value))) => finish_unhandled(
            result,
            Some(target),
            UiAccessibilityActionStatus::Rejected,
            "mutation_rejected",
            report
                .message
                .as_deref()
                .unwrap_or("adjust value mutation was rejected"),
        ),
        Ok(None) => unsupported_role_action(result, target, "target has no range value contract"),
        Err(error) => finish_unhandled(
            result,
            Some(target),
            UiAccessibilityActionStatus::Rejected,
            "mutation_error",
            &format!("adjust value mutation failed: {error}"),
        ),
    }
}

fn reject_missing_target(
    surface: &UiSurface,
    snapshot: &UiAccessibilityTreeSnapshot,
    target: UiNodeId,
    mut result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    if !surface.tree.nodes.contains_key(&target) {
        return finish_unhandled(
            result,
            None,
            UiAccessibilityActionStatus::StaleTarget,
            "stale_target",
            "target is not in the runtime UI tree",
        );
    }

    append_target_diagnostics(snapshot, target, &mut result);
    if is_effectively_hidden(surface, target) {
        return finish_unhandled(
            result,
            Some(target),
            UiAccessibilityActionStatus::Rejected,
            "hidden_target",
            "target is hidden and excluded from accessibility action dispatch",
        );
    }

    finish_unhandled(
        result,
        Some(target),
        UiAccessibilityActionStatus::Rejected,
        "excluded_target",
        "target is not included in the current accessibility snapshot",
    )
}

fn dispatch_focus(
    surface: &mut UiSurface,
    target: UiNodeId,
    snapshot_node: &UiAccessibilityNode,
    result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    if !snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Focus)
    {
        return unsupported_role_action(result, target, "target does not expose focus action");
    }

    match surface.focus_node_with_reason(
        target,
        UiFocusChangeReason::Programmatic,
        UiFocusVisible::visible(UiFocusVisibleReason::Programmatic),
    ) {
        Ok(_) => finish_handled(result, target, "accessibility.focus"),
        Err(error) => finish_unhandled(
            result,
            Some(target),
            UiAccessibilityActionStatus::Rejected,
            "focus_rejected",
            &format!("focus target rejected by runtime focus API: {error}"),
        ),
    }
}

fn dispatch_activate(
    surface: &mut UiSurface,
    target: UiNodeId,
    snapshot_node: &UiAccessibilityNode,
    result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    if !snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Activate)
    {
        return unsupported_role_action(result, target, "target does not expose activate action");
    }

    match surface.apply_default_keyboard_component_action(target) {
        Ok(report) if report.handled => {
            let mut result = finish_handled(result, target, "accessibility.activate");
            result.component_events.extend(report.component_events);
            return result;
        }
        Ok(_) => {}
        Err(error) => {
            return finish_unhandled(
                result,
                Some(target),
                UiAccessibilityActionStatus::Rejected,
                "mutation_error",
                &format!("activate widget action failed: {error}"),
            );
        }
    }

    let mut result = finish_handled(result, target, "accessibility.activate");
    result.component_events.push(UiComponentEventReport {
        target,
        event: UiComponentEvent::Commit {
            property: "activated".to_string(),
            value: UiValue::Bool(true),
        },
        delivered: true,
    });
    result
}

fn dispatch_set_value(
    surface: &mut UiSurface,
    request: &zircon_runtime_interface::ui::accessibility::UiAccessibilityActionRequest,
    snapshot_node: &UiAccessibilityNode,
    result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    let target = request.target;
    if !snapshot_node
        .actions
        .contains(&UiAccessibilityAction::SetValue)
    {
        return unsupported_role_action(result, target, "target does not expose set value action");
    }
    if !matches!(
        snapshot_node.role,
        UiA11yRole::TextInput | UiA11yRole::Slider
    ) {
        return unsupported_role_action(
            result,
            target,
            "set value requires text input or slider role",
        );
    }
    let Some(property) = set_value_property(surface, target) else {
        return unsupported_role_action(
            result,
            target,
            "target has no mutable value or text property",
        );
    };
    let Some(value) = set_value_payload(request, snapshot_node.role) else {
        return finish_unhandled(
            result,
            Some(target),
            UiAccessibilityActionStatus::Rejected,
            "missing_value",
            "set value action requires value or numeric_value",
        );
    };

    let report = surface.mutate_property(
        UiPropertyMutationRequest::new(target, property, value.clone())
            .with_source(UiReflectedPropertySource::RuntimeState),
    );
    match report {
        Ok(report) if matches!(report.status, UiPropertyMutationStatus::Accepted) => {
            let mut result = finish_handled(result, target, "accessibility.set_value");
            result.diagnostics.notes.push(format!(
                "accessibility_property_changed:{}:{:?}",
                report.property, report.invalidation.dirty
            ));
            result.component_events.push(UiComponentEventReport {
                target,
                event: UiComponentEvent::ValueChanged {
                    property: report.property,
                    value,
                },
                delivered: true,
            });
            result
        }
        Ok(report) if matches!(report.status, UiPropertyMutationStatus::Unchanged) => {
            let mut result = finish_handled(result, target, "accessibility.set_value");
            result.diagnostics.notes.push(format!(
                "accessibility_property_unchanged:{}",
                report.property
            ));
            result
        }
        Ok(report) => finish_unhandled(
            result,
            Some(target),
            UiAccessibilityActionStatus::Rejected,
            "mutation_rejected",
            report
                .message
                .as_deref()
                .unwrap_or("set value mutation was rejected"),
        ),
        Err(error) => finish_unhandled(
            result,
            Some(target),
            UiAccessibilityActionStatus::Rejected,
            "mutation_error",
            &format!("set value mutation failed: {error}"),
        ),
    }
}

fn set_value_property(surface: &UiSurface, target: UiNodeId) -> Option<String> {
    let metadata = surface
        .tree
        .nodes
        .get(&target)?
        .template_metadata
        .as_ref()?;
    let attributes = &metadata.attributes;
    if let Some(property) = metadata.widget.value_property.as_deref() {
        return (attributes.contains_key(property)
            || surface
                .component_states
                .get(target)
                .and_then(|state| state.value(property))
                .is_some()
            || metadata.widget.value.is_some())
        .then(|| property.to_string());
    }
    if attributes.contains_key("value") {
        Some("value".to_string())
    } else if attributes.contains_key("text") {
        Some("text".to_string())
    } else {
        None
    }
}

fn set_value_payload(
    request: &zircon_runtime_interface::ui::accessibility::UiAccessibilityActionRequest,
    role: UiA11yRole,
) -> Option<UiValue> {
    match role {
        UiA11yRole::TextInput => request.value.clone().map(UiValue::String).or_else(|| {
            request
                .numeric_value
                .map(|value| UiValue::String(value.to_string()))
        }),
        UiA11yRole::Slider => request
            .numeric_value
            .filter(|value| value.is_finite())
            .map(UiValue::Float)
            .or_else(|| {
                request
                    .value
                    .as_deref()
                    .and_then(|value| value.parse::<f64>().ok())
                    .filter(|value| value.is_finite())
                    .map(UiValue::Float)
            }),
        _ => None,
    }
}

fn unsupported_role_action(
    result: UiInputDispatchResult,
    target: UiNodeId,
    reason: &str,
) -> UiInputDispatchResult {
    finish_unhandled(
        result,
        Some(target),
        UiAccessibilityActionStatus::Unsupported,
        "unsupported_role_action",
        reason,
    )
}

fn finish_handled(
    mut result: UiInputDispatchResult,
    target: UiNodeId,
    phase: &str,
) -> UiInputDispatchResult {
    result.reply = UiDispatchReply::handled().from_handler(target);
    result.diagnostics.routed = true;
    result.diagnostics.route_target = Some(target);
    result.diagnostics.handled_phase = Some(phase.to_string());
    result.diagnostics.notes.push(action_note(
        UiAccessibilityActionStatus::Accepted,
        None,
        None,
    ));
    result
}

fn finish_unhandled(
    mut result: UiInputDispatchResult,
    route_target: Option<UiNodeId>,
    status: UiAccessibilityActionStatus,
    code: &str,
    reason: &str,
) -> UiInputDispatchResult {
    result.diagnostics.route_target = route_target;
    result
        .diagnostics
        .notes
        .push(action_note(status, Some(code), Some(reason)));
    result
}

fn finish_unhandled_with_note(
    result: UiInputDispatchResult,
    route_target: Option<UiNodeId>,
    status: UiAccessibilityActionStatus,
    code: &str,
    reason: &str,
    note: &str,
) -> UiInputDispatchResult {
    let mut result = finish_unhandled(result, route_target, status, code, reason);
    result.diagnostics.notes.push(note.to_string());
    result
}

fn action_note(
    status: UiAccessibilityActionStatus,
    code: Option<&str>,
    reason: Option<&str>,
) -> String {
    let mut note = format!("status={}", status_label(status));
    if let Some(code) = code {
        note.push_str(" code=");
        note.push_str(code);
    }
    if let Some(reason) = reason {
        note.push_str(" reason=");
        note.push_str(reason);
    }
    note
}

fn status_label(status: UiAccessibilityActionStatus) -> &'static str {
    match status {
        UiAccessibilityActionStatus::Accepted => "accepted",
        UiAccessibilityActionStatus::Rejected => "rejected",
        UiAccessibilityActionStatus::Unsupported => "unsupported",
        UiAccessibilityActionStatus::StaleTarget => "stale_target",
    }
}

fn append_target_diagnostics(
    snapshot: &UiAccessibilityTreeSnapshot,
    target: UiNodeId,
    result: &mut UiInputDispatchResult,
) {
    result.diagnostics.notes.extend(
        snapshot
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.node_id == Some(target))
            .map(|diagnostic| format!("accessibility_diagnostic={:?}", diagnostic.code)),
    );
}

fn is_effectively_hidden(surface: &UiSurface, target: UiNodeId) -> bool {
    let mut current = Some(target);
    while let Some(node_id) = current {
        let Some(node) = surface.tree.nodes.get(&node_id) else {
            return false;
        };
        if !node.is_render_visible() {
            return true;
        }
        current = node.parent;
    }
    false
}
