use zircon_runtime_interface::ui::{
    binding::{UiBindingSourceKind, UiEventKind},
    component::UiValue,
    dispatch::{
        UiComponentEventReport, UiDispatchReply, UiInputDispatchResult, UiInputEvent,
        UiNumberInputCommitMethod, UiNumberInputCommitStatus,
    },
    event_ui::UiNodeId,
    surface::{UiEditableTextState, UiTextCaret, UiTextCaretAffinity, UiTextEditAction},
    text::UiTextEditSource,
    widget::UiWidgetEvent,
};

use crate::ui::{
    dispatch::UiTextDocumentSession,
    surface::UiTextComponentEventKind,
    text::{CommittedTextEditIntent, apply_text_edit_action_with_intent},
};

use super::super::super::surface::UiSurface;
use super::super::{
    effect::append_dispatch_effect_to_result,
    owner_route::{focused_input_kind_for_event, record_owner_focused_input},
    text_state::{
        editable_text_input_is_secure, editable_text_state_for_node, editable_value_property,
        is_number_field_metadata,
    },
};
use super::ime_context::input_method_update_for_text_state;
use super::{
    document_transaction::{
        PreparedUiEditableTextDocumentTransaction, UiEditableTextDocumentTransactionReceipt,
    },
    property_transaction::{
        commit_editable_text_properties_with_edit, prepare_editable_text_properties_with_edit,
        prepare_number_field_properties,
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui) enum UiEditableTextTransactionError {
    Property(super::property_transaction::UiEditableTextPropertyTransactionError),
    Document(crate::ui::dispatch::UiTextDocumentSessionError),
}

impl UiEditableTextTransactionError {
    pub(in crate::ui) const fn diagnostic_code(self) -> &'static str {
        match self {
            Self::Property(error) => error.diagnostic_code(),
            Self::Document(error) => error.diagnostic_code(),
        }
    }
}

pub(in crate::ui::surface::input) fn apply_editable_text_state(
    surface: &mut UiSurface,
    text_documents: Option<&mut UiTextDocumentSession>,
    event: UiInputEvent,
    target: UiNodeId,
    state: UiEditableTextState,
    committed_edit: Option<CommittedTextEditIntent>,
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
    let transaction = match commit_editable_text_transaction(
        surface,
        text_documents,
        target,
        value_property.as_str(),
        &state,
        UiBindingSourceKind::WidgetBehavior,
        committed_edit,
        &mut result,
    ) {
        Ok(transaction) => transaction,
        Err(error) => {
            result.diagnostics.notes.push(format!(
                "text_state_transaction_rejected:{}",
                error.diagnostic_code()
            ));
            surface.redact_secure_text_result(target, &mut result);
            return result;
        }
    };
    let number_input = transaction.number_input;
    let number_publish_value = transaction.number_publish_value;
    let value_changed = transaction.value_changed;
    if let Some(binding_report) = transaction.binding_report {
        result.record_binding_report(binding_report);
    }
    for (property, dirty) in transaction.changed_properties {
        result
            .diagnostics
            .notes
            .push(format!("text_property_changed:{property}:{dirty:?}"));
    }
    if transaction.committed_edit.is_some() {
        result
            .diagnostics
            .notes
            .push("text_edit_intent_prepared".to_string());
    }
    if let Some(number_input) = number_input {
        result.diagnostics.number_input = Some(number_input);
        if let Some(value) = number_publish_value {
            push_component_event_report(
                surface,
                target,
                value_property.as_str(),
                UiValue::Float(value),
                state.text.len(),
                TextComponentEventKind::Change,
                value_changed,
                &mut result,
            );
        }
    } else {
        push_component_event_report(
            surface,
            target,
            value_property.as_str(),
            UiValue::String(state.text.clone()),
            state.text.len(),
            component_event_kind,
            value_changed,
            &mut result,
        );
    }
    if let Some(effect) = input_method_update_for_text_state(surface, &result.event, target, &state)
    {
        append_dispatch_effect_to_result(surface, &mut result, effect);
    }
    surface.redact_secure_text_result(target, &mut result);

    result
}

#[allow(clippy::too_many_arguments)]
pub(in crate::ui) fn commit_editable_text_transaction(
    surface: &mut UiSurface,
    text_documents: Option<&mut UiTextDocumentSession>,
    target: UiNodeId,
    value_property: &str,
    state: &UiEditableTextState,
    source_kind: UiBindingSourceKind,
    committed_edit: Option<CommittedTextEditIntent>,
    result: &mut UiInputDispatchResult,
) -> Result<
    super::property_transaction::UiEditableTextPropertyTransactionReceipt,
    UiEditableTextTransactionError,
> {
    let Some(intent) = committed_edit else {
        return commit_editable_text_properties_with_edit(
            surface,
            target,
            value_property,
            state,
            source_kind,
            None,
        )
        .map_err(UiEditableTextTransactionError::Property);
    };
    let Some(text_documents) = text_documents else {
        result
            .diagnostics
            .notes
            .push("text_document_session_unavailable".to_string());
        return commit_editable_text_properties_with_edit(
            surface,
            target,
            value_property,
            state,
            source_kind,
            Some(intent),
        )
        .map_err(UiEditableTextTransactionError::Property);
    };

    let tree_id = surface.tree.tree_id.clone();
    let Some(source_epoch) = surface.input.text_document_epoch(target) else {
        return Err(UiEditableTextTransactionError::Document(
            crate::ui::dispatch::UiTextDocumentSessionError::SourceEpochExhausted,
        ));
    };
    let current_state = editable_text_state_for_node(surface, target).ok_or(
        UiEditableTextTransactionError::Document(
            crate::ui::dispatch::UiTextDocumentSessionError::InvalidEditIntent,
        ),
    )?;
    let history_commit = text_documents
        .prepare_history_commit(
            &tree_id,
            target,
            source_epoch,
            &intent,
            &current_state,
            state,
            editable_text_input_is_secure(surface, target),
        )
        .map_err(UiEditableTextTransactionError::Document)?;
    let document = text_documents
        .prepare_edit(
            &tree_id,
            target,
            source_epoch,
            &intent,
            state,
            text_edit_source(&result.event),
        )
        .map_err(UiEditableTextTransactionError::Document)?;
    let properties = prepare_editable_text_properties_with_edit(
        surface,
        target,
        value_property,
        state,
        source_kind,
        Some(intent),
    )
    .map_err(UiEditableTextTransactionError::Property)?;
    let UiEditableTextDocumentTransactionReceipt {
        properties,
        document,
    } = PreparedUiEditableTextDocumentTransaction::new(properties, document)
        .commit()
        .map_err(UiEditableTextTransactionError::Property)?;
    let next_source_epoch = source_epoch + 1;
    debug_assert_eq!(
        surface.input.text_document_epoch(target),
        Some(next_source_epoch)
    );
    text_documents.finish_edit(
        &tree_id,
        target,
        next_source_epoch,
        &document,
        history_commit,
    );
    if let crate::text::document::TextDocumentStoreEditCommit::Changed { public_receipt, .. } =
        document
    {
        result.widget_events.push(UiWidgetEvent::TextEditChange {
            receipt: Box::new(public_receipt),
        });
    }
    Ok(properties)
}

const fn text_edit_source(event: &UiInputEvent) -> UiTextEditSource {
    match event {
        UiInputEvent::Pointer(_) => UiTextEditSource::Pointer,
        UiInputEvent::Ime(_) => UiTextEditSource::Ime,
        UiInputEvent::Clipboard(_) => UiTextEditSource::Clipboard,
        UiInputEvent::Accessibility(_) => UiTextEditSource::Accessibility,
        UiInputEvent::Keyboard(_) | UiInputEvent::Text(_) => UiTextEditSource::Keyboard,
        _ => UiTextEditSource::Programmatic,
    }
}

pub(in crate::ui::surface::input) fn submit_editable_text_state(
    surface: &mut UiSurface,
    event: UiInputEvent,
    target: UiNodeId,
    state: UiEditableTextState,
    repeated: bool,
) -> UiInputDispatchResult {
    let kind = focused_input_kind_for_event(&event);
    let mut result = UiInputDispatchResult::new(event, UiDispatchReply::handled());
    result.diagnostics.routed = true;
    result.diagnostics.route_target = Some(target);
    result.diagnostics.handled_phase = Some("keyboard.submit".to_string());
    if let Some(kind) = kind {
        record_owner_focused_input(surface, kind, target, Some(target), true);
    }
    if repeated {
        return result;
    }

    let Some(value_property) = editable_value_property(surface, target) else {
        result
            .diagnostics
            .notes
            .push("editable value property missing".to_string());
        return result;
    };
    if is_number_field(surface, target) {
        apply_number_field_commit_to_result(
            surface,
            target,
            value_property.as_str(),
            &state,
            UiNumberInputCommitMethod::Enter,
            &mut result,
        );
        surface.redact_secure_text_result(target, &mut result);
        return result;
    }
    push_component_event_report(
        surface,
        target,
        value_property.as_str(),
        UiValue::String(state.text.clone()),
        state.text.len(),
        TextComponentEventKind::Submit,
        false,
        &mut result,
    );
    surface.redact_secure_text_result(target, &mut result);
    result
}

pub(in crate::ui::surface) fn finish_editable_text_for_focus_loss(
    surface: &mut UiSurface,
    target: UiNodeId,
) -> Option<UiComponentEventReport> {
    let Some(editable) = editable_text_state_for_node(surface, target) else {
        return None;
    };
    let publishes_commit = !editable.read_only;
    let restored = cancel_editable_text_composition(surface, target, editable)?;
    if !publishes_commit {
        return None;
    }
    let value_property = editable_value_property(surface, target)?;
    if is_number_field(surface, target) {
        return commit_number_field_state(
            surface,
            target,
            value_property.as_str(),
            &restored,
            UiNumberInputCommitMethod::FocusLoss,
        )
        .and_then(|commit| commit.reports.into_iter().next());
    }
    focus_loss_commit_event(surface, target, value_property, restored)
}

pub(in crate::ui::surface::input) fn cancel_number_field_edit_state(
    surface: &mut UiSurface,
    event: UiInputEvent,
    target: UiNodeId,
    state: UiEditableTextState,
) -> UiInputDispatchResult {
    let mut result = UiInputDispatchResult::new(event, UiDispatchReply::handled());
    result.diagnostics.routed = true;
    result.diagnostics.route_target = Some(target);
    result.diagnostics.handled_phase = Some("keyboard.number_cancel".to_string());
    let Some(value_property) = editable_value_property(surface, target) else {
        result
            .diagnostics
            .notes
            .push("editable value property missing".to_string());
        return result;
    };
    apply_number_field_commit_to_result(
        surface,
        target,
        value_property.as_str(),
        &state,
        UiNumberInputCommitMethod::Escape,
        &mut result,
    );
    surface.redact_secure_text_result(target, &mut result);
    result
}

pub(in crate::ui::surface::input) fn step_number_field_keyboard_state(
    surface: &mut UiSurface,
    event: UiInputEvent,
    target: UiNodeId,
    state: &UiEditableTextState,
    direction: f64,
) -> Option<UiInputDispatchResult> {
    let decision = super::super::number_field::number_field_keyboard_step_decision(
        surface, target, direction,
    )?;
    let kind = focused_input_kind_for_event(&event);
    let mut result = UiInputDispatchResult::new(event, UiDispatchReply::handled());
    result.diagnostics.routed = true;
    result.diagnostics.route_target = Some(target);
    result.diagnostics.handled_phase = Some("keyboard.number_step".to_string());
    if let Some(kind) = kind {
        record_owner_focused_input(surface, kind, target, Some(target), true);
    }
    let Some(value_property) = editable_value_property(surface, target) else {
        result
            .diagnostics
            .notes
            .push("editable value property missing".to_string());
        return Some(result);
    };
    if decision.receipt.commit_status == UiNumberInputCommitStatus::Rejected {
        result.diagnostics.number_input = Some(decision.receipt);
        return Some(result);
    }
    apply_number_field_decision_to_result(
        surface,
        target,
        value_property.as_str(),
        state,
        decision,
        &mut result,
    );
    surface.redact_secure_text_result(target, &mut result);
    Some(result)
}

pub(in crate::ui::surface) fn cancel_editable_text_composition_for_input_method_loss(
    surface: &mut UiSurface,
    target: UiNodeId,
) {
    let Some(editable) = editable_text_state_for_node(surface, target) else {
        return;
    };
    let _ = cancel_editable_text_composition(surface, target, editable);
}

fn cancel_editable_text_composition(
    surface: &mut UiSurface,
    target: UiNodeId,
    editable: UiEditableTextState,
) -> Option<UiEditableTextState> {
    if editable.composition.is_none() {
        return Some(editable);
    }
    let restored =
        apply_text_edit_action_with_intent(editable, UiTextEditAction::CancelComposition).state;
    let value_property = editable_value_property(surface, target)?;
    commit_editable_text_properties_with_edit(
        surface,
        target,
        value_property.as_str(),
        &restored,
        UiBindingSourceKind::WidgetBehavior,
        None,
    )
    .ok()?;
    Some(restored)
}

fn focus_loss_commit_event(
    surface: &mut UiSurface,
    target: UiNodeId,
    value_property: String,
    state: UiEditableTextState,
) -> Option<UiComponentEventReport> {
    text_component_event_reports(
        surface,
        target,
        value_property,
        UiValue::String(state.text.clone()),
        TextComponentEventKind::Submit,
        None,
    )
    .into_iter()
    .next()
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::ui::surface::input) enum TextComponentEventKind {
    Change,
    Submit,
}

fn push_component_event_report(
    surface: &mut UiSurface,
    target: UiNodeId,
    value_property: &str,
    value: UiValue,
    payload_bytes: usize,
    kind: TextComponentEventKind,
    value_changed: bool,
    result: &mut UiInputDispatchResult,
) {
    if kind == TextComponentEventKind::Change && !value_changed {
        return;
    }
    let reports = text_component_event_reports(
        surface,
        target,
        value_property.to_string(),
        value,
        kind,
        result.drag,
    );
    if !reports.is_empty() {
        super::profile::record_component_payload(payload_bytes);
        result.component_events.extend(reports);
    }
}

fn text_component_event_reports(
    surface: &mut UiSurface,
    target: UiNodeId,
    value_property: String,
    value: UiValue,
    kind: TextComponentEventKind,
    drag: Option<zircon_runtime_interface::ui::component::UiDragMetrics>,
) -> Vec<UiComponentEventReport> {
    let binding_event = match kind {
        TextComponentEventKind::Change => UiEventKind::Change,
        TextComponentEventKind::Submit => UiEventKind::Submit,
    };
    let template_actions =
        if let Some(sources) = surface.compiled_binding_event_sources(target, binding_event) {
            sources
                .iter()
                .filter_map(|source| {
                    let handle = source.handle;
                    let binding = surface.compiled_bindings.binding(handle)?;
                    (binding.event == binding_event).then(|| {
                        binding
                            .targets
                            .is_empty()
                            .then(|| {
                                surface.template_action_for_compiled_binding_with_overrides(
                                    target,
                                    handle,
                                    std::collections::BTreeMap::new(),
                                )
                            })
                            .flatten()
                    })
                })
                .collect::<Vec<_>>()
        } else {
            surface
                .tree
                .nodes
                .get(&target)
                .and_then(|node| node.template_metadata.as_ref())
                .map(|metadata| {
                    metadata
                        .bindings
                        .iter()
                        .filter(|binding| binding.event == binding_event)
                        .map(|binding| {
                            binding
                                .targets
                                .is_empty()
                                .then(|| surface.template_action_for_binding(target, binding))
                                .flatten()
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        };
    if template_actions.is_empty() {
        return Vec::new();
    }

    let event_kind = match kind {
        TextComponentEventKind::Change => UiTextComponentEventKind::Change,
        TextComponentEventKind::Submit => UiTextComponentEventKind::Commit,
    };
    let event = surface.component_value_event(target, value_property, value, event_kind);

    template_actions
        .into_iter()
        .map(|template_action| UiComponentEventReport {
            target,
            event: event.clone(),
            delivered: true,
            drag,
            template_action,
        })
        .collect()
}

struct NumberFieldCommit {
    receipt: zircon_runtime_interface::ui::dispatch::UiNumberInputReceiptV1,
    reports: Vec<UiComponentEventReport>,
    transaction: super::property_transaction::UiEditableTextPropertyTransactionReceipt,
}

fn apply_number_field_commit_to_result(
    surface: &mut UiSurface,
    target: UiNodeId,
    value_property: &str,
    state: &UiEditableTextState,
    method: UiNumberInputCommitMethod,
    result: &mut UiInputDispatchResult,
) {
    let Some(decision) = super::super::number_field::number_field_commit_decision(
        surface,
        target,
        &state.text,
        method,
    ) else {
        result
            .diagnostics
            .notes
            .push("number_field_commit_rejected:missing_or_invalid_metadata".to_string());
        return;
    };
    apply_number_field_decision_to_result(surface, target, value_property, state, decision, result);
}

fn apply_number_field_decision_to_result(
    surface: &mut UiSurface,
    target: UiNodeId,
    value_property: &str,
    state: &UiEditableTextState,
    decision: super::super::number_field::NumberFieldCommitDecision,
    result: &mut UiInputDispatchResult,
) {
    let Some(commit) =
        commit_number_field_decision(surface, target, value_property, state, decision)
    else {
        result
            .diagnostics
            .notes
            .push("number_field_commit_rejected:property_transaction".to_string());
        return;
    };
    result.diagnostics.number_input = Some(commit.receipt);
    if let Some(binding_report) = commit.transaction.binding_report {
        result.record_binding_report(binding_report);
    }
    for (property, dirty) in commit.transaction.changed_properties {
        result
            .diagnostics
            .notes
            .push(format!("number_property_changed:{property}:{dirty:?}"));
    }
    result.component_events.extend(commit.reports);
}

fn commit_number_field_state(
    surface: &mut UiSurface,
    target: UiNodeId,
    value_property: &str,
    state: &UiEditableTextState,
    method: UiNumberInputCommitMethod,
) -> Option<NumberFieldCommit> {
    let decision = super::super::number_field::number_field_commit_decision(
        surface,
        target,
        &state.text,
        method,
    )?;
    commit_number_field_decision(surface, target, value_property, state, decision)
}

fn commit_number_field_decision(
    surface: &mut UiSurface,
    target: UiNodeId,
    value_property: &str,
    state: &UiEditableTextState,
    decision: super::super::number_field::NumberFieldCommitDecision,
) -> Option<NumberFieldCommit> {
    let committed_state = UiEditableTextState {
        text: decision.text.clone(),
        caret: UiTextCaret {
            offset: decision.text.len(),
            affinity: UiTextCaretAffinity::Downstream,
        },
        selection: None,
        composition: None,
        read_only: state.read_only,
    };
    let transaction = prepare_number_field_properties(
        surface,
        target,
        value_property,
        UiValue::Float(decision.value),
        &committed_state,
        decision.edit_active,
        UiBindingSourceKind::WidgetBehavior,
    )
    .ok()?
    .commit()
    .ok()?;
    let publish = matches!(
        decision.receipt.commit_status,
        UiNumberInputCommitStatus::Applied
            | UiNumberInputCommitStatus::Unchanged
            | UiNumberInputCommitStatus::Clamped
            | UiNumberInputCommitStatus::Snapped
    );
    let reports = if publish {
        text_component_event_reports(
            surface,
            target,
            value_property.to_string(),
            UiValue::Float(decision.value),
            TextComponentEventKind::Submit,
            None,
        )
    } else {
        Vec::new()
    };
    Some(NumberFieldCommit {
        receipt: decision.receipt,
        reports,
        transaction,
    })
}

fn is_number_field(surface: &UiSurface, target: UiNodeId) -> bool {
    surface
        .tree
        .node(target)
        .and_then(|node| node.template_metadata.as_ref())
        .is_some_and(is_number_field_metadata)
}
