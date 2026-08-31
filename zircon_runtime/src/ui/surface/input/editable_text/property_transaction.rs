use zircon_runtime_interface::ui::{
    binding::{
        UiBindingDirtyDomain, UiBindingSourceKind, UiBindingUpdateReport, UiBindingUpdateStatus,
    },
    component::UiValue,
    dispatch::UiNumberInputReceiptV1,
    event_ui::UiNodeId,
    surface::{UiEditableTextState, UiTextCaretAffinity},
    tree::UiDirtyFlags,
};

use crate::ui::{
    binding::component_state_value_update_with_source_kind,
    editable_text_composition::composition_clauses_value,
    surface::{
        UiSurface, input::is_number_field_metadata,
        property_mutation::mutate_tree_metadata_properties,
    },
    text::{CommittedTextEditIntent, clamp_grapheme_boundary},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui) enum UiEditableTextPropertyTransactionError {
    MissingNode,
    MissingMetadata,
    ReservedValueProperty,
    ValueKindMismatch,
    InvalidState,
    NumberRevisionExhausted,
    InvalidEditIntent,
}

impl UiEditableTextPropertyTransactionError {
    pub(in crate::ui) const fn diagnostic_code(self) -> &'static str {
        match self {
            Self::MissingNode => "missing_node",
            Self::MissingMetadata => "missing_metadata",
            Self::ReservedValueProperty => "reserved_value_property",
            Self::ValueKindMismatch => "value_kind_mismatch",
            Self::InvalidState => "invalid_state",
            Self::NumberRevisionExhausted => "number_revision_exhausted",
            Self::InvalidEditIntent => "invalid_edit_intent",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(in crate::ui) struct UiEditableTextPropertyTransactionReceipt {
    pub(in crate::ui) value_changed: bool,
    pub(in crate::ui) text_changed: bool,
    pub(in crate::ui) changed_properties: Vec<(String, UiDirtyFlags)>,
    pub(in crate::ui) binding_report: Option<UiBindingUpdateReport>,
    pub(in crate::ui) dirty: UiDirtyFlags,
    pub(in crate::ui) committed_edit: Option<CommittedTextEditIntent>,
    pub(in crate::ui) number_input: Option<UiNumberInputReceiptV1>,
    pub(in crate::ui) number_publish_value: Option<f64>,
}

#[must_use = "prepared editable text properties must be committed or explicitly discarded"]
pub(in crate::ui) struct PreparedUiEditableTextPropertyTransaction<'surface> {
    surface: &'surface mut UiSurface,
    target: UiNodeId,
    value_property: String,
    text_property: String,
    properties: [(String, UiValue); 10],
    supplemental_properties: [Option<(String, UiValue)>; 4],
    source_kind: UiBindingSourceKind,
    committed_edit: Option<CommittedTextEditIntent>,
    number_input: Option<UiNumberInputReceiptV1>,
    number_publish_value: Option<f64>,
}

impl PreparedUiEditableTextPropertyTransaction<'_> {
    pub(in crate::ui) fn commit(
        self,
    ) -> Result<UiEditableTextPropertyTransactionReceipt, UiEditableTextPropertyTransactionError>
    {
        crate::profile_scope!("runtime", "ui_text.edit", "property_commit");
        let Self {
            surface,
            target,
            value_property,
            text_property,
            properties,
            supplemental_properties,
            source_kind,
            committed_edit,
            number_input,
            number_publish_value,
        } = self;
        let batch = mutate_tree_metadata_properties(
            &mut surface.tree,
            target,
            properties
                .into_iter()
                .chain(supplemental_properties.into_iter().flatten()),
            source_kind,
        )
        .map_err(|_| UiEditableTextPropertyTransactionError::MissingNode)?;
        if batch.changes.is_empty() {
            return Ok(UiEditableTextPropertyTransactionReceipt {
                committed_edit,
                number_input,
                number_publish_value,
                ..Default::default()
            });
        }

        let value_changed = batch
            .changes
            .iter()
            .any(|change| change.property == value_property);
        let text_changed = batch
            .changes
            .iter()
            .any(|change| change.property == text_property);
        let mut combined_dirty = batch.dirty;
        let mut binding_updates = Vec::with_capacity(batch.changes.len() * 2);
        let mut changed_properties = Vec::with_capacity(batch.changes.len());
        for (change, mut reflected_update) in batch.changes.into_iter().zip(batch.reflected_updates)
        {
            let previous_component_value =
                surface
                    .component_states
                    .get(target)
                    .and_then(|component_state| {
                        component_state.value(change.property.as_str()).cloned()
                    });
            let _ = surface.runtime_style.set_base_attribute(
                target,
                change.property.clone(),
                change.value.to_toml(),
            );
            let component_change = surface.component_states.sync_from_property(
                target,
                change.property.as_str(),
                &change.value,
            );
            debug_assert!(!component_change.pseudo_state_changed);

            let mut dirty = change.dirty;
            if change.property == value_property || change.property == text_property {
                dirty.layout = true;
                dirty.render = true;
                dirty.text = true;
            }
            if component_change.any_changed() {
                dirty.render = true;
            }
            reflected_update.dirty = UiBindingDirtyDomain::from_dirty_flags(dirty);
            merge_dirty_flags(&mut combined_dirty, dirty);
            binding_updates.push(reflected_update);
            if component_change.any_changed() {
                binding_updates.push(component_state_value_update_with_source_kind(
                    target,
                    change.property.clone(),
                    source_kind,
                    previous_component_value,
                    change.value,
                    dirty,
                    UiBindingUpdateStatus::Applied,
                ));
            }
            changed_properties.push((change.property, dirty));
        }

        surface
            .mark_node_dirty(target, combined_dirty)
            .map_err(|_| UiEditableTextPropertyTransactionError::MissingNode)?;
        if text_changed {
            surface.input.advance_text_document_epoch(target);
        }
        surface.invalidate_clipboard_transfers_for(target);
        Ok(UiEditableTextPropertyTransactionReceipt {
            value_changed,
            text_changed,
            changed_properties,
            binding_report: Some(UiBindingUpdateReport::from_updates(binding_updates)),
            dirty: combined_dirty,
            committed_edit,
            number_input,
            number_publish_value,
        })
    }
}

pub(in crate::ui) fn commit_editable_text_properties(
    surface: &mut UiSurface,
    target: UiNodeId,
    value_property: &str,
    state: &UiEditableTextState,
    source_kind: UiBindingSourceKind,
) -> Result<UiEditableTextPropertyTransactionReceipt, UiEditableTextPropertyTransactionError> {
    commit_editable_text_properties_with_edit(
        surface,
        target,
        value_property,
        state,
        source_kind,
        None,
    )
}

pub(in crate::ui) fn commit_editable_text_properties_with_edit(
    surface: &mut UiSurface,
    target: UiNodeId,
    value_property: &str,
    state: &UiEditableTextState,
    source_kind: UiBindingSourceKind,
    committed_edit: Option<CommittedTextEditIntent>,
) -> Result<UiEditableTextPropertyTransactionReceipt, UiEditableTextPropertyTransactionError> {
    prepare_editable_text_properties_with_edit(
        surface,
        target,
        value_property,
        state,
        source_kind,
        committed_edit,
    )?
    .commit()
}

pub(in crate::ui) fn prepare_editable_text_properties_with_edit<'surface>(
    surface: &'surface mut UiSurface,
    target: UiNodeId,
    value_property: &str,
    state: &UiEditableTextState,
    source_kind: UiBindingSourceKind,
    committed_edit: Option<CommittedTextEditIntent>,
) -> Result<
    PreparedUiEditableTextPropertyTransaction<'surface>,
    UiEditableTextPropertyTransactionError,
> {
    crate::profile_scope!("runtime", "ui_text.edit", "property_prepare");
    super::profile::record_property_value_clone(state.text.len());
    let is_number_field = surface
        .tree
        .node(target)
        .and_then(|node| node.template_metadata.as_ref())
        .is_some_and(is_number_field_metadata);
    let prepared = if is_number_field {
        let number_edit =
            super::super::number_field::number_field_edit_decision(surface, target, &state.text);
        let canonical_value = number_edit
            .and_then(|decision| decision.publish_value)
            .map(UiValue::Float)
            .map_or_else(|| canonical_value(surface, target, value_property), Ok)?;
        prepare_number_field_properties_with_edit(
            surface,
            target,
            value_property,
            canonical_value,
            state,
            true,
            false,
            source_kind,
            number_edit,
            committed_edit,
        )?
    } else {
        prepare_editable_text_properties_with_values_and_edit(
            surface,
            target,
            value_property,
            UiValue::String(state.text.clone()),
            value_property,
            UiValue::String(state.text.clone()),
            [None, None, None],
            None,
            state,
            source_kind,
            committed_edit,
        )?
    };
    super::profile::record_property_projection(
        state.text.len(),
        prepared.committed_edit.is_some(),
        state.composition.is_some(),
        state
            .composition
            .as_ref()
            .map_or(0, |composition| composition.text.len()),
    );
    Ok(prepared)
}

pub(in crate::ui) fn commit_editable_text_properties_with_value(
    surface: &mut UiSurface,
    target: UiNodeId,
    value_property: &str,
    value: UiValue,
    state: &UiEditableTextState,
    source_kind: UiBindingSourceKind,
) -> Result<UiEditableTextPropertyTransactionReceipt, UiEditableTextPropertyTransactionError> {
    prepare_editable_text_properties_with_value(
        surface,
        target,
        value_property,
        value,
        state,
        source_kind,
    )?
    .commit()
}

pub(in crate::ui) fn prepare_editable_text_properties_with_value<'surface>(
    surface: &'surface mut UiSurface,
    target: UiNodeId,
    value_property: &str,
    value: UiValue,
    state: &UiEditableTextState,
    source_kind: UiBindingSourceKind,
) -> Result<
    PreparedUiEditableTextPropertyTransaction<'surface>,
    UiEditableTextPropertyTransactionError,
> {
    if surface
        .tree
        .node(target)
        .and_then(|node| node.template_metadata.as_ref())
        .is_some_and(is_number_field_metadata)
    {
        prepare_number_field_properties(
            surface,
            target,
            value_property,
            value,
            state,
            false,
            source_kind,
        )
    } else {
        prepare_editable_text_properties_with_values_and_edit(
            surface,
            target,
            value_property,
            value.clone(),
            value_property,
            value,
            [None, None, None],
            None,
            state,
            source_kind,
            None,
        )
    }
}

pub(in crate::ui) fn prepare_number_field_properties<'surface>(
    surface: &'surface mut UiSurface,
    target: UiNodeId,
    value_property: &str,
    value: UiValue,
    state: &UiEditableTextState,
    edit_active: bool,
    source_kind: UiBindingSourceKind,
) -> Result<
    PreparedUiEditableTextPropertyTransaction<'surface>,
    UiEditableTextPropertyTransactionError,
> {
    prepare_number_field_properties_with_edit(
        surface,
        target,
        value_property,
        value,
        state,
        edit_active,
        false,
        source_kind,
        None,
        None,
    )
}

pub(in crate::ui) fn prepare_number_field_model_update_properties<'surface>(
    surface: &'surface mut UiSurface,
    target: UiNodeId,
    value_property: &str,
    value: UiValue,
    state: &UiEditableTextState,
    edit_active: bool,
    preserve_edit_base: bool,
) -> Result<
    PreparedUiEditableTextPropertyTransaction<'surface>,
    UiEditableTextPropertyTransactionError,
> {
    prepare_number_field_properties_with_edit(
        surface,
        target,
        value_property,
        value,
        state,
        edit_active,
        preserve_edit_base,
        UiBindingSourceKind::RuntimeState,
        None,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
fn prepare_number_field_properties_with_edit<'surface>(
    surface: &'surface mut UiSurface,
    target: UiNodeId,
    value_property: &str,
    value: UiValue,
    state: &UiEditableTextState,
    edit_active: bool,
    preserve_edit_base: bool,
    source_kind: UiBindingSourceKind,
    number_edit: Option<super::super::number_field::NumberFieldEditDecision>,
    committed_edit: Option<CommittedTextEditIntent>,
) -> Result<
    PreparedUiEditableTextPropertyTransaction<'surface>,
    UiEditableTextPropertyTransactionError,
> {
    let revisions = super::super::number_field::number_field_revision_projection(
        surface,
        target,
        &value,
        edit_active,
        preserve_edit_base,
    )
    .map_err(|error| match error {
        super::super::number_field::NumberFieldRevisionError::InvalidState => {
            UiEditableTextPropertyTransactionError::InvalidState
        }
        super::super::number_field::NumberFieldRevisionError::Exhausted => {
            UiEditableTextPropertyTransactionError::NumberRevisionExhausted
        }
    })?;
    prepare_editable_text_properties_with_values_and_edit(
        surface,
        target,
        value_property,
        value,
        "value_text",
        UiValue::String(state.text.clone()),
        [
            Some(("number_edit_active".to_string(), UiValue::Bool(edit_active))),
            Some((
                super::super::number_field::NUMBER_FIELD_VALUE_REVISION_PROPERTY.to_string(),
                UiValue::Int(revisions.value_revision),
            )),
            Some((
                super::super::number_field::NUMBER_FIELD_EDIT_BASE_REVISION_PROPERTY.to_string(),
                UiValue::Int(revisions.edit_base_revision),
            )),
        ],
        number_edit,
        state,
        source_kind,
        committed_edit,
    )
}

#[allow(clippy::too_many_arguments)]
fn prepare_editable_text_properties_with_values_and_edit<'surface>(
    surface: &'surface mut UiSurface,
    target: UiNodeId,
    value_property: &str,
    value: UiValue,
    text_property: &str,
    text_value: UiValue,
    additional_properties: [Option<(String, UiValue)>; 3],
    number_edit: Option<super::super::number_field::NumberFieldEditDecision>,
    state: &UiEditableTextState,
    source_kind: UiBindingSourceKind,
    committed_edit: Option<CommittedTextEditIntent>,
) -> Result<
    PreparedUiEditableTextPropertyTransaction<'surface>,
    UiEditableTextPropertyTransactionError,
> {
    prepare_editable_text_property_transaction(
        surface,
        target,
        value_property,
        &value,
        text_property,
        &text_value,
        &additional_properties,
        state,
        committed_edit.as_ref(),
    )?;
    Ok(PreparedUiEditableTextPropertyTransaction {
        surface,
        target,
        value_property: value_property.to_string(),
        text_property: text_property.to_string(),
        properties: editable_text_properties(text_property, text_value, state),
        supplemental_properties: [
            (value_property != text_property).then(|| (value_property.to_string(), value)),
            additional_properties[0].clone(),
            additional_properties[1].clone(),
            additional_properties[2].clone(),
        ],
        source_kind,
        committed_edit,
        number_input: number_edit.map(|decision| decision.receipt),
        number_publish_value: number_edit.and_then(|decision| decision.publish_value),
    })
}

fn prepare_editable_text_property_transaction(
    surface: &UiSurface,
    target: UiNodeId,
    value_property: &str,
    value: &UiValue,
    text_property: &str,
    text_value: &UiValue,
    additional_properties: &[Option<(String, UiValue)>; 3],
    state: &UiEditableTextState,
    committed_edit: Option<&CommittedTextEditIntent>,
) -> Result<(), UiEditableTextPropertyTransactionError> {
    let node = surface
        .tree
        .node(target)
        .ok_or(UiEditableTextPropertyTransactionError::MissingNode)?;
    let metadata = node
        .template_metadata
        .as_ref()
        .ok_or(UiEditableTextPropertyTransactionError::MissingMetadata)?;
    if value_property.is_empty() || value_property_is_reserved(value_property) {
        return Err(UiEditableTextPropertyTransactionError::ReservedValueProperty);
    }
    let kind_matches = |property: &str, proposed: &UiValue| {
        metadata
            .attributes
            .get(property)
            .map(UiValue::from_toml)
            .is_none_or(|current| {
                std::mem::discriminant(&current) == std::mem::discriminant(proposed)
            })
    };
    if !kind_matches(value_property, value)
        || !kind_matches(text_property, text_value)
        || additional_properties
            .iter()
            .flatten()
            .any(|(property, value)| !kind_matches(property, value))
    {
        return Err(UiEditableTextPropertyTransactionError::ValueKindMismatch);
    }
    if text_value.display_text() != state.text || !editable_text_state_is_valid(state) {
        return Err(UiEditableTextPropertyTransactionError::InvalidState);
    }
    if committed_edit.is_some_and(|intent| !intent.is_valid_for_state(state)) {
        return Err(UiEditableTextPropertyTransactionError::InvalidEditIntent);
    }
    Ok(())
}

fn canonical_value(
    surface: &UiSurface,
    target: UiNodeId,
    value_property: &str,
) -> Result<UiValue, UiEditableTextPropertyTransactionError> {
    surface
        .tree
        .node(target)
        .and_then(|node| node.template_metadata.as_ref())
        .and_then(|metadata| metadata.attributes.get(value_property))
        .map(UiValue::from_toml)
        .ok_or(UiEditableTextPropertyTransactionError::MissingMetadata)
}

fn editable_text_properties(
    value_property: &str,
    value: UiValue,
    state: &UiEditableTextState,
) -> [(String, UiValue); 10] {
    let (selection_anchor, selection_focus) = state
        .selection
        .as_ref()
        .map(|selection| (selection.anchor, selection.focus))
        .unwrap_or((state.caret.offset, state.caret.offset));
    let (composition_start, composition_end, composition_text, restore_text, preedit_clauses) =
        state
            .composition
            .as_ref()
            .map(|composition| {
                (
                    composition.range.start,
                    composition.range.end,
                    composition.text.clone(),
                    composition.restore_text.clone().unwrap_or_default(),
                    composition_clauses_value(&composition.preedit_clauses),
                )
            })
            .unwrap_or((
                state.caret.offset,
                state.caret.offset,
                String::new(),
                String::new(),
                UiValue::Array(Vec::new()),
            ));

    [
        (value_property.to_string(), value),
        (
            "caret_offset".to_string(),
            UiValue::Int(state.caret.offset as i64),
        ),
        (
            "caret_affinity".to_string(),
            caret_affinity_property_value(state.caret.affinity),
        ),
        (
            "selection_anchor".to_string(),
            UiValue::Int(selection_anchor as i64),
        ),
        (
            "selection_focus".to_string(),
            UiValue::Int(selection_focus as i64),
        ),
        (
            "composition_start".to_string(),
            UiValue::Int(composition_start as i64),
        ),
        (
            "composition_end".to_string(),
            UiValue::Int(composition_end as i64),
        ),
        (
            "composition_text".to_string(),
            UiValue::String(composition_text),
        ),
        (
            "composition_restore_text".to_string(),
            UiValue::String(restore_text),
        ),
        ("composition_clauses".to_string(), preedit_clauses),
    ]
}

fn editable_text_state_is_valid(state: &UiEditableTextState) -> bool {
    let valid_offset = |offset| {
        offset <= state.text.len() && clamp_grapheme_boundary(&state.text, offset) == offset
    };
    if !valid_offset(state.caret.offset) {
        return false;
    }
    if state
        .selection
        .as_ref()
        .is_some_and(|selection| !valid_offset(selection.anchor) || !valid_offset(selection.focus))
    {
        return false;
    }
    !state.composition.as_ref().is_some_and(|composition| {
        composition.range.start > composition.range.end
            || !valid_offset(composition.range.start)
            || !valid_offset(composition.range.end)
    })
}

fn value_property_is_reserved(property: &str) -> bool {
    matches!(
        property,
        "visibility"
            | "enabled"
            | "disabled"
            | "visible"
            | "clickable"
            | "hoverable"
            | "focusable"
            | "pressed"
            | "checked"
            | "input_policy"
            | "open"
            | "popup_open"
            | "caret_offset"
            | "caret_affinity"
            | "selection_anchor"
            | "selection_focus"
            | "composition_start"
            | "composition_end"
            | "composition_text"
            | "composition_restore_text"
            | "composition_clauses"
            | "read_only"
            | "readOnly"
            | "input_read_only"
            | "inputReadOnly"
            | "secure"
            | "secure_input"
            | "secureInput"
            | "password"
            | "input_kind"
            | "inputKind"
            | "type"
            | "editable_text"
            | "editableText"
            | "multiline"
            | "max_graphemes"
            | "max_chars"
            | "max_length"
            | "input_filter"
            | "text_filter"
    )
}

fn caret_affinity_property_value(affinity: UiTextCaretAffinity) -> UiValue {
    UiValue::String(
        match affinity {
            UiTextCaretAffinity::Downstream => "downstream",
            UiTextCaretAffinity::Upstream => "upstream",
        }
        .to_string(),
    )
}

fn merge_dirty_flags(target: &mut UiDirtyFlags, dirty: UiDirtyFlags) {
    target.layout |= dirty.layout;
    target.hit_test |= dirty.hit_test;
    target.render |= dirty.render;
    target.style |= dirty.style;
    target.text |= dirty.text;
    target.input |= dirty.input;
    target.visible_range |= dirty.visible_range;
}
