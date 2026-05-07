use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    component::{UiComponentEvent, UiValue},
    dispatch::{
        UiComponentEventReport, UiDispatchAppliedEffect, UiDispatchDisposition, UiDispatchEffect,
        UiDispatchReply, UiDragDropEffectKind, UiDragDropInputEvent, UiDragDropInputEventKind,
        UiImeInputEvent, UiImeInputEventKind, UiInputDispatchResult, UiInputEvent,
        UiNavigationInputEvent, UiPointerId, UiPopupEffectKind, UiPopupInputEvent,
        UiPopupInputEventKind, UiTextByteRange, UiTextInputEvent, UiTooltipEffectKind,
        UiTooltipTimerInputEvent, UiTooltipTimerInputEventKind,
    },
    event_ui::{UiNodeId, UiReflectedPropertySource},
    surface::{
        UiEditableTextState, UiPointerEventKind, UiTextCaret, UiTextCaretAffinity,
        UiTextComposition, UiTextEditAction, UiTextRange, UiTextSelection,
    },
    tree::UiTreeError,
};

use crate::ui::dispatch::{UiNavigationDispatcher, UiPointerDispatcher};
use crate::ui::surface::{UiPropertyMutationRequest, UiPropertyMutationStatus};
use crate::ui::text::apply_text_edit_action;
use crate::ui::tree::UiRuntimeTreeRoutingExt;

use super::super::surface::UiSurface;
use super::{apply_dispatch_reply, is_valid_input_owner};

pub(crate) fn dispatch_input_event(
    surface: &mut UiSurface,
    pointer_dispatcher: &UiPointerDispatcher,
    navigation_dispatcher: &UiNavigationDispatcher,
    event: UiInputEvent,
) -> Result<UiInputDispatchResult, UiTreeError> {
    match event {
        UiInputEvent::Pointer(pointer) => {
            let metadata = pointer.metadata.clone();
            let legacy =
                surface.dispatch_pointer_event(pointer_dispatcher, pointer.event.clone())?;
            let event = UiInputEvent::Pointer(pointer);
            if legacy.captured_by.is_some() {
                surface.input.captured_pointer_id = metadata.pointer_id;
            }
            if legacy.diagnostics.capture_released {
                if let Some(owner) = legacy.released_capture.or(legacy.route.captured) {
                    surface.input.clear_pointer_capture_for(owner);
                } else {
                    surface.input.clear_pointer_capture();
                }
            }
            let reply = pointer_reply(&legacy, metadata.pointer_id.unwrap_or_default());
            let mut applied_effects = Vec::new();
            for (effect_index, effect) in reply.effects.iter().cloned().enumerate() {
                applied_effects.push(UiDispatchAppliedEffect {
                    effect_index,
                    effect,
                });
            }
            let mut result = UiInputDispatchResult::new(event, reply.clone());
            result.diagnostics.routed = legacy.diagnostics.pointer_routed;
            result.diagnostics.route_target = legacy.route.target;
            result.diagnostics.blocked_by = legacy.blocked_by;
            result.diagnostics.handled_phase = if legacy.handled_by.is_some() {
                Some("pointer".to_string())
            } else {
                None
            };
            if legacy.route.kind == UiPointerEventKind::Scroll {
                result
                    .diagnostics
                    .notes
                    .push(format!("scroll_delta={}", legacy.route.scroll_delta));
            }
            result.applied_effects = applied_effects;
            result.component_events = legacy
                .component_events
                .into_iter()
                .map(|event| UiComponentEventReport {
                    target: event.node_id,
                    event: event.envelope.event,
                    delivered: true,
                })
                .collect();
            Ok(result)
        }
        UiInputEvent::Navigation(navigation) => {
            dispatch_navigation_input(surface, navigation_dispatcher, navigation)
        }
        UiInputEvent::Keyboard(keyboard) => {
            let target = surface.focus.focused;
            let mut result = UiInputDispatchResult::new(
                UiInputEvent::Keyboard(keyboard),
                UiDispatchReply::unhandled(),
            );
            result.diagnostics.routed = target.is_some();
            result.diagnostics.route_target = target;
            if let Some(target) = target {
                result
                    .diagnostics
                    .notes
                    .push(format_route(surface, target)?);
            }
            Ok(result)
        }
        UiInputEvent::Text(text) => Ok(dispatch_text_input(surface, text)),
        UiInputEvent::Ime(ime) => Ok(dispatch_ime_input(surface, ime)),
        UiInputEvent::Analog(analog) => {
            let changed = surface
                .input
                .update_analog_control(analog.control.as_str(), analog.value);
            let mut result = owner_routed_result(
                surface,
                UiInputEvent::Analog(analog),
                surface.focus.focused,
                "analog.focused",
            );
            if !changed {
                result.reply = UiDispatchReply::unhandled();
                result.diagnostics.routed = false;
                result
                    .diagnostics
                    .notes
                    .push("analog_repeat_suppressed".to_string());
            }
            Ok(result)
        }
        UiInputEvent::DragDrop(drag_drop) => Ok(dispatch_drag_drop_input(surface, drag_drop)),
        UiInputEvent::Popup(popup) => Ok(dispatch_popup_input(surface, popup)),
        UiInputEvent::TooltipTimer(tooltip) => Ok(dispatch_tooltip_timer_input(surface, tooltip)),
    }
}

fn dispatch_navigation_input(
    surface: &mut UiSurface,
    dispatcher: &UiNavigationDispatcher,
    navigation: UiNavigationInputEvent,
) -> Result<UiInputDispatchResult, UiTreeError> {
    let legacy = surface.dispatch_navigation_event(dispatcher, navigation.kind)?;
    let event = UiInputEvent::Navigation(navigation);
    let mut reply = if legacy.handled_by.is_some() || legacy.focus_changed_to.is_some() {
        UiDispatchReply::handled()
    } else {
        UiDispatchReply::unhandled()
    };
    if let Some(target) = legacy.focus_changed_to {
        reply = reply.with_effect(UiDispatchEffect::SetFocus {
            target,
            reason: zircon_runtime_interface::ui::dispatch::UiFocusEffectReason::Navigation,
        });
    }
    let mut result = UiInputDispatchResult::new(event, reply.clone());
    result.diagnostics.routed = legacy.route.target.is_some() || legacy.route.fallback_to_root;
    result.diagnostics.route_target = legacy.route.target.or(legacy.focus_changed_to);
    result.diagnostics.handled_phase = legacy.handled_by.map(|_| "navigation".to_string());
    for (effect_index, effect) in reply.effects.into_iter().enumerate() {
        result.applied_effects.push(UiDispatchAppliedEffect {
            effect_index,
            effect,
        });
    }
    Ok(result)
}

fn pointer_reply(
    legacy: &zircon_runtime_interface::ui::dispatch::UiPointerDispatchResult,
    pointer_id: UiPointerId,
) -> UiDispatchReply {
    let disposition = if legacy.blocked_by.is_some() {
        UiDispatchDisposition::Blocked
    } else if legacy.handled_by.is_some() {
        UiDispatchDisposition::Handled
    } else if !legacy.passthrough.is_empty() {
        UiDispatchDisposition::Passthrough
    } else {
        UiDispatchDisposition::Unhandled
    };
    let mut reply = UiDispatchReply {
        disposition,
        handler: legacy.handled_by.or(legacy.blocked_by),
        phase: Some(zircon_runtime_interface::ui::dispatch::UiDispatchPhase::Bubble),
        effects: Vec::new(),
    };
    if let Some(target) = legacy.captured_by {
        reply.effects.push(UiDispatchEffect::CapturePointer {
            target,
            pointer_id,
            reason: zircon_runtime_interface::ui::dispatch::UiPointerCaptureReason::Press,
        });
    }
    reply
}

fn dispatch_drag_drop_input(
    surface: &mut UiSurface,
    drag_drop: UiDragDropInputEvent,
) -> UiInputDispatchResult {
    let target = drag_drop_target(surface, drag_drop.point);
    let event = UiInputEvent::DragDrop(drag_drop.clone());
    let Some(target) = target else {
        let mut result = UiInputDispatchResult::new(event, UiDispatchReply::unhandled());
        result
            .diagnostics
            .notes
            .push("drag_drop target missing".to_string());
        return result;
    };

    let effect_kind = match drag_drop.kind {
        UiDragDropInputEventKind::Begin => UiDragDropEffectKind::Begin,
        UiDragDropInputEventKind::Enter | UiDragDropInputEventKind::Over => {
            UiDragDropEffectKind::Update
        }
        UiDragDropInputEventKind::Leave => UiDragDropEffectKind::Reject,
        UiDragDropInputEventKind::Drop => UiDragDropEffectKind::Accept,
        UiDragDropInputEventKind::End => UiDragDropEffectKind::Complete,
    };
    let pointer_id = drag_drop.metadata.pointer_id.unwrap_or_default();
    let reply = UiDispatchReply::handled().with_effect(UiDispatchEffect::DragDrop {
        kind: effect_kind,
        target,
        pointer_id,
        session_id: drag_drop.session_id,
        point: Some(drag_drop.point),
        payload: drag_drop.payload.clone(),
    });
    let mut result = apply_dispatch_reply(surface, event, reply);
    result.diagnostics.routed = result.rejected_effects.is_empty();
    result.diagnostics.route_target = Some(target);
    result.diagnostics.handled_phase = Some("drag_drop.effect".to_string());
    result
}

fn drag_drop_target(
    surface: &UiSurface,
    point: zircon_runtime_interface::ui::layout::UiPoint,
) -> Option<zircon_runtime_interface::ui::event_ui::UiNodeId> {
    surface
        .hit_test(point)
        .top_hit
        .or_else(|| surface.input.drag_drop.as_ref().map(|drag| drag.target))
        .or(surface.focus.captured)
        .or(surface.focus.focused)
}

fn dispatch_popup_input(
    surface: &mut UiSurface,
    popup: UiPopupInputEvent,
) -> UiInputDispatchResult {
    let effect_kind = match popup.kind {
        UiPopupInputEventKind::OpenRequested => UiPopupEffectKind::Open,
        UiPopupInputEventKind::CloseRequested | UiPopupInputEventKind::Dismissed => {
            UiPopupEffectKind::Close
        }
    };
    let event = UiInputEvent::Popup(popup.clone());
    let reply = UiDispatchReply::handled().with_effect(UiDispatchEffect::Popup {
        kind: effect_kind,
        popup_id: popup.popup_id,
        anchor: popup.anchor,
    });
    let mut result = apply_dispatch_reply(surface, event, reply);
    result.diagnostics.routed = result.rejected_effects.is_empty();
    result.diagnostics.handled_phase = Some("popup.effect".to_string());
    result
}

fn dispatch_tooltip_timer_input(
    surface: &mut UiSurface,
    tooltip: UiTooltipTimerInputEvent,
) -> UiInputDispatchResult {
    let effect_kind = match tooltip.kind {
        UiTooltipTimerInputEventKind::Armed => UiTooltipEffectKind::Arm,
        UiTooltipTimerInputEventKind::Elapsed => UiTooltipEffectKind::Show,
        UiTooltipTimerInputEventKind::Canceled => UiTooltipEffectKind::Cancel,
    };
    let event = UiInputEvent::TooltipTimer(tooltip.clone());
    let reply = UiDispatchReply::handled().with_effect(UiDispatchEffect::Tooltip {
        kind: effect_kind,
        tooltip_id: tooltip.tooltip_id,
    });
    let mut result = apply_dispatch_reply(surface, event, reply);
    result.diagnostics.routed = result.rejected_effects.is_empty();
    result.diagnostics.handled_phase = Some("tooltip.effect".to_string());
    result
}

fn dispatch_text_input(surface: &mut UiSurface, text: UiTextInputEvent) -> UiInputDispatchResult {
    let target = text_input_target(surface);
    let event = UiInputEvent::Text(text.clone());
    let Some(target) = target else {
        return owner_routed_result(surface, event, None, "text.owner");
    };

    let Some(editable) = editable_text_state_for_node(surface, target) else {
        let mut result = owner_routed_result(surface, event, Some(target), "text.owner");
        result
            .diagnostics
            .notes
            .push("text target is not editable".to_string());
        return result;
    };

    let next = committed_text_state(editable, text.text);
    apply_editable_text_state(
        surface,
        event,
        target,
        next,
        "text.edit",
        TextComponentEventKind::Change,
    )
}

fn dispatch_ime_input(surface: &mut UiSurface, ime: UiImeInputEvent) -> UiInputDispatchResult {
    let target = surface.input.input_method_owner;
    let clear_owner = matches!(ime.kind, UiImeInputEventKind::Cancel);
    let event = UiInputEvent::Ime(ime.clone());
    let Some(target) = target.filter(|owner| is_valid_input_owner(surface, *owner)) else {
        surface.input.clear_input_method();
        let mut result = owner_routed_result(surface, event, None, "ime.owner");
        result
            .diagnostics
            .notes
            .push("owner route rejected".to_string());
        result
            .diagnostics
            .notes
            .push("ime owner missing".to_string());
        return result;
    };

    let Some(editable) = editable_text_state_for_node(surface, target) else {
        let mut result = owner_routed_result(surface, event, Some(target), "ime.owner");
        result
            .diagnostics
            .notes
            .push("ime target is not editable".to_string());
        if clear_owner {
            surface.input.clear_input_method();
            result
                .diagnostics
                .notes
                .push("ime owner cleared".to_string());
        }
        return result;
    };

    let component_event_kind = match ime.kind {
        UiImeInputEventKind::Commit => TextComponentEventKind::Submit,
        _ => TextComponentEventKind::Change,
    };
    let next = match ime.kind {
        UiImeInputEventKind::Preedit => preedit_text_state(editable, &ime.text, ime.cursor_range),
        UiImeInputEventKind::Commit => committed_text_state(editable, ime.text),
        UiImeInputEventKind::Cancel => {
            apply_text_edit_action(editable, UiTextEditAction::CancelComposition)
        }
    };

    let mut result = apply_editable_text_state(
        surface,
        event,
        target,
        next,
        "ime.edit",
        component_event_kind,
    );
    if clear_owner {
        surface.input.clear_input_method();
        result
            .diagnostics
            .notes
            .push("ime owner cleared".to_string());
    }
    result
}

fn text_input_target(surface: &mut UiSurface) -> Option<UiNodeId> {
    let ime_owner = surface.input.input_method_owner;
    if ime_owner.is_some_and(|owner| is_valid_input_owner(surface, owner)) {
        return ime_owner;
    }
    if ime_owner.is_some() {
        surface.input.clear_input_method();
    }
    surface
        .focus
        .focused
        .filter(|owner| is_valid_input_owner(surface, *owner))
}

fn committed_text_state(editable: UiEditableTextState, text: String) -> UiEditableTextState {
    if editable.composition.is_some() {
        let range = editable
            .composition
            .as_ref()
            .map(|composition| composition.range)
            .unwrap_or(UiTextRange {
                start: editable.caret.offset,
                end: editable.caret.offset,
            });
        let composed =
            apply_text_edit_action(editable, UiTextEditAction::SetComposition { range, text });
        apply_text_edit_action(composed, UiTextEditAction::CommitComposition)
    } else {
        apply_text_edit_action(editable, UiTextEditAction::Insert { text })
    }
}

fn preedit_text_state(
    editable: UiEditableTextState,
    preedit: &str,
    cursor_range: Option<UiTextByteRange>,
) -> UiEditableTextState {
    let range = editable
        .composition
        .as_ref()
        .map(|composition| composition.range)
        .or_else(|| editable.selection.as_ref().map(UiTextSelection::range))
        .unwrap_or(UiTextRange {
            start: editable.caret.offset,
            end: editable.caret.offset,
        });
    let mut next = apply_text_edit_action(
        editable,
        UiTextEditAction::SetComposition {
            range,
            text: preedit.to_string(),
        },
    );

    if let Some(cursor_range) = cursor_range {
        if let Some(composition) = next.composition.as_ref() {
            let anchor = composition.range.start + cursor_range.start_byte as usize;
            let focus = composition.range.start + cursor_range.end_byte as usize;
            next = if anchor == focus {
                apply_text_edit_action(
                    next,
                    UiTextEditAction::MoveCaret {
                        offset: focus,
                        extend_selection: false,
                    },
                )
            } else {
                apply_text_edit_action(next, UiTextEditAction::SetSelection { anchor, focus })
            };
        }
    }

    next
}

fn editable_text_state_for_node(
    surface: &UiSurface,
    target: UiNodeId,
) -> Option<UiEditableTextState> {
    let metadata = surface
        .tree
        .nodes
        .get(&target)?
        .template_metadata
        .as_ref()?;
    if !is_editable_text_component(metadata) {
        return None;
    }
    let property = editable_value_property(surface, target)?;
    let text = string_attribute(metadata, property)
        .or_else(|| string_attribute(metadata, "value_text"))
        .or_else(|| string_attribute(metadata, "text"))
        .unwrap_or_default();
    let caret_offset = usize_attribute(metadata, "caret_offset").unwrap_or(text.len());
    let selection = usize_attribute(metadata, "selection_anchor")
        .zip(usize_attribute(metadata, "selection_focus"))
        .map(|(anchor, focus)| UiTextSelection {
            anchor: clamp_text_boundary(&text, anchor),
            focus: clamp_text_boundary(&text, focus),
        });
    let composition = usize_attribute(metadata, "composition_start")
        .zip(usize_attribute(metadata, "composition_end"))
        .zip(string_attribute(metadata, "composition_text"))
        .map(|((start, end), composition_text)| UiTextComposition {
            range: UiTextRange {
                start: clamp_text_boundary(&text, start),
                end: clamp_text_boundary(&text, end),
            },
            text: composition_text,
            restore_text: string_attribute(metadata, "composition_restore_text"),
        });

    Some(UiEditableTextState {
        caret: UiTextCaret {
            offset: clamp_text_boundary(&text, caret_offset),
            affinity: UiTextCaretAffinity::Downstream,
        },
        selection,
        composition,
        read_only: bool_attribute(metadata, "read_only")
            .or_else(|| bool_attribute(metadata, "input_read_only"))
            .unwrap_or(false),
        text,
    })
}

fn apply_editable_text_state(
    surface: &mut UiSurface,
    event: UiInputEvent,
    target: UiNodeId,
    state: UiEditableTextState,
    phase: &str,
    component_event_kind: TextComponentEventKind,
) -> UiInputDispatchResult {
    let mut result = UiInputDispatchResult::new(event, UiDispatchReply::handled());
    result.diagnostics.routed = true;
    result.diagnostics.route_target = Some(target);
    result.diagnostics.handled_phase = Some(phase.to_string());

    let Some(value_property) = editable_value_property(surface, target) else {
        result
            .diagnostics
            .notes
            .push("editable value property missing".to_string());
        return result;
    };

    mutate_text_property(
        surface,
        target,
        value_property,
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
        value_property,
        &state,
        component_event_kind,
        &mut result,
    );

    result
}

#[derive(Clone, Copy)]
enum TextComponentEventKind {
    Change,
    Submit,
}

fn push_text_component_event_report(
    surface: &UiSurface,
    target: UiNodeId,
    value_property: &str,
    state: &UiEditableTextState,
    kind: TextComponentEventKind,
    result: &mut UiInputDispatchResult,
) {
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
    });
}

fn mutate_text_property(
    surface: &mut UiSurface,
    target: UiNodeId,
    property: &str,
    value: UiValue,
    result: &mut UiInputDispatchResult,
) {
    let report = surface.mutate_property(
        UiPropertyMutationRequest::new(target, property, value)
            .with_source(UiReflectedPropertySource::RuntimeState),
    );
    match report {
        Ok(report) => {
            if matches!(report.status, UiPropertyMutationStatus::Accepted) {
                result.diagnostics.notes.push(format!(
                    "text_property_changed:{}:{:?}",
                    report.property, report.invalidation.dirty
                ));
            }
        }
        Err(error) => result
            .diagnostics
            .notes
            .push(format!("text_property_rejected:{property}:{error}")),
    }
}

fn editable_value_property(surface: &UiSurface, target: UiNodeId) -> Option<&'static str> {
    let metadata = surface
        .tree
        .nodes
        .get(&target)?
        .template_metadata
        .as_ref()?;
    if metadata.attributes.contains_key("value") {
        Some("value")
    } else if metadata.attributes.contains_key("text") {
        Some("text")
    } else {
        Some("value")
    }
}

fn is_editable_text_component(
    metadata: &zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata,
) -> bool {
    bool_attribute(metadata, "editable_text").unwrap_or(false)
        || matches!(
            metadata.component.as_str(),
            "InputField" | "TextField" | "LineEdit" | "TextEdit" | "NumberField"
        )
}

fn string_attribute(
    metadata: &zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata,
    key: &str,
) -> Option<String> {
    metadata.attributes.get(key).and_then(|value| match value {
        toml::Value::String(value) => Some(value.clone()),
        toml::Value::Integer(value) => Some(value.to_string()),
        toml::Value::Float(value) => Some(value.to_string()),
        toml::Value::Boolean(value) => Some(value.to_string()),
        _ => None,
    })
}

fn usize_attribute(
    metadata: &zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata,
    key: &str,
) -> Option<usize> {
    metadata.attributes.get(key).and_then(|value| match value {
        toml::Value::Integer(value) => (*value >= 0).then_some(*value as usize),
        toml::Value::Float(value) if value.is_finite() && *value >= 0.0 => Some(*value as usize),
        _ => None,
    })
}

fn bool_attribute(
    metadata: &zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata,
    key: &str,
) -> Option<bool> {
    metadata.attributes.get(key).and_then(toml::Value::as_bool)
}

fn clamp_text_boundary(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

fn owner_routed_result(
    surface: &UiSurface,
    event: UiInputEvent,
    target: Option<zircon_runtime_interface::ui::event_ui::UiNodeId>,
    phase: &str,
) -> UiInputDispatchResult {
    let valid_target = target.filter(|node_id| is_valid_input_owner(surface, *node_id));
    let reply = if valid_target.is_some() {
        UiDispatchReply::handled()
    } else {
        UiDispatchReply::unhandled()
    };
    let mut result = UiInputDispatchResult::new(event, reply);
    result.diagnostics.routed = valid_target.is_some();
    result.diagnostics.route_target = valid_target;
    result.diagnostics.handled_phase = valid_target.map(|_| phase.to_string());
    if target.is_some() && valid_target.is_none() {
        result
            .diagnostics
            .notes
            .push("owner route rejected".to_string());
    }
    result
}

fn format_route(
    surface: &UiSurface,
    target: zircon_runtime_interface::ui::event_ui::UiNodeId,
) -> Result<String, UiTreeError> {
    let route = surface.tree.bubble_route(target)?;
    Ok(format!("focused_route_len={}", route.len()))
}
