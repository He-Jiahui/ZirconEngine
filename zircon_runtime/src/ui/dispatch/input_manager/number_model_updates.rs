use std::collections::BTreeMap;

use zircon_runtime_interface::ui::{
    component::UiValue,
    event_ui::UiNodeId,
    surface::{UiEditableTextState, UiTextCaret, UiTextCaretAffinity},
    text::{
        UI_NUMBER_MODEL_UPDATE_SCHEMA_VERSION, UiNumberModelId, UiNumberModelKey,
        UiNumberModelRevision, UiNumberModelUpdateFailure, UiNumberModelUpdateId,
        UiNumberModelUpdateOrigin, UiNumberModelUpdateReceipt, UiNumberModelUpdateRequest,
        UiNumberModelUpdateStatus,
    },
};

use crate::ui::surface::{
    UiSurface, UiSurfaceSessionIdentityHandle,
    input::{
        NumberFieldRevisionError, UiEditableTextPropertyTransactionError,
        editable_text_state_for_node, editable_value_property, is_number_field_metadata,
        number_field_edit_is_active, number_field_value_revision,
        prepare_number_field_model_update_properties,
    },
};

use super::manager::UiInputManager;

const NUMBER_MODEL_PROFILE_COUNTER_NAMES: [&str; 5] = [
    "number_model_update_request_count",
    "number_model_update_focused_buffer_preserved_count",
    "number_model_update_conflict_count",
    "number_model_update_revision_advance_count",
    "number_model_update_rejected_count",
];

impl UiInputManager {
    /// Returns the manager-owned identity and canonical revision for a NumberField owner.
    pub fn number_model_key(
        &mut self,
        surface: &mut UiSurface,
        owner: UiNodeId,
    ) -> Result<UiNumberModelKey, UiNumberModelUpdateFailure> {
        self.synchronize_text_document_owners(surface);
        self.number_model_updates.model_key(surface, owner)
    }

    /// Applies one revision-qualified bound refresh or explicit NumberField replacement.
    pub fn update_number_model(
        &mut self,
        surface: &mut UiSurface,
        request: UiNumberModelUpdateRequest,
    ) -> UiNumberModelUpdateReceipt {
        crate::profile_scope!("runtime", "ui_text.number_model_update", "apply_request");
        crate::profile_counter!("runtime", NUMBER_MODEL_PROFILE_COUNTER_NAMES[0], 1);
        self.synchronize_text_document_owners(surface);
        apply_request(&mut self.number_model_updates, surface, request)
    }
}

#[derive(Default)]
pub(super) struct UiNumberModelUpdateState {
    active_surface: Option<UiSurfaceSessionIdentityHandle>,
    observed_layout_order_generation: Option<u64>,
    model_ids: BTreeMap<UiNodeId, UiNumberModelIdentity>,
}

#[derive(Clone, Copy)]
struct UiNumberModelIdentity {
    node_incarnation: u64,
    model_id: UiNumberModelId,
}

impl UiNumberModelUpdateState {
    pub(super) fn activate_surface(&mut self, surface: &UiSurface) {
        let identity = surface.session_identity();
        if self.active_surface.as_ref() == Some(&identity) {
            return;
        }
        self.active_surface = Some(identity);
        self.observed_layout_order_generation = Some(surface.tree.layout_order_generation());
        self.model_ids.clear();
    }

    pub(super) fn reconcile_owners(&mut self, surface: &UiSurface) {
        let layout_order_generation = surface.tree.layout_order_generation();
        if self.observed_layout_order_generation != Some(layout_order_generation) {
            // One topology change can replace an owner while reusing its UiNodeId between syncs.
            self.observed_layout_order_generation = Some(layout_order_generation);
            self.model_ids.retain(|owner, _| {
                surface
                    .tree
                    .node(*owner)
                    .and_then(|node| node.template_metadata.as_ref())
                    .is_some_and(is_number_field_metadata)
            });
        }
    }

    fn model_key(
        &mut self,
        surface: &UiSurface,
        owner: UiNodeId,
    ) -> Result<UiNumberModelKey, UiNumberModelUpdateFailure> {
        let revision = number_field_value_revision(surface, owner).map_err(map_revision_error)?;
        let node_incarnation = surface
            .tree
            .node_incarnation(owner)
            .ok_or(UiNumberModelUpdateFailure::InvalidTarget)?;
        let identity = self
            .model_ids
            .entry(owner)
            .or_insert_with(|| UiNumberModelIdentity {
                node_incarnation,
                model_id: UiNumberModelId::issue(),
            });
        if identity.node_incarnation != node_incarnation {
            *identity = UiNumberModelIdentity {
                node_incarnation,
                model_id: UiNumberModelId::issue(),
            };
        }
        Ok(UiNumberModelKey {
            model_id: identity.model_id,
            revision: UiNumberModelRevision::new(revision as u64),
        })
    }
}

#[derive(Clone)]
struct UiNumberModelUpdateEnvelope {
    request_id: UiNumberModelUpdateId,
    tree_id: zircon_runtime_interface::ui::event_ui::UiTreeId,
    node_id: UiNodeId,
    expected_model: UiNumberModelKey,
    origin: UiNumberModelUpdateOrigin,
}

impl From<&UiNumberModelUpdateRequest> for UiNumberModelUpdateEnvelope {
    fn from(request: &UiNumberModelUpdateRequest) -> Self {
        Self {
            request_id: request.request_id,
            tree_id: request.tree_id.clone(),
            node_id: request.node_id,
            expected_model: request.expected_model,
            origin: request.origin,
        }
    }
}

fn apply_request(
    state: &mut UiNumberModelUpdateState,
    surface: &mut UiSurface,
    request: UiNumberModelUpdateRequest,
) -> UiNumberModelUpdateReceipt {
    let envelope = UiNumberModelUpdateEnvelope::from(&request);
    if let Err(failure) = request.validate() {
        return rejected(&envelope, None, failure);
    }
    if request.tree_id != surface.tree.tree_id {
        return rejected(&envelope, None, UiNumberModelUpdateFailure::WrongTree);
    }
    let current_model = match state.model_key(surface, request.node_id) {
        Ok(key) => key,
        Err(failure) => return rejected(&envelope, None, failure),
    };
    if request.expected_model != current_model {
        crate::profile_counter!("runtime", NUMBER_MODEL_PROFILE_COUNTER_NAMES[2], 1);
        return receipt(
            &envelope,
            UiNumberModelUpdateStatus::Conflict,
            Some(current_model),
            Some(UiNumberModelUpdateFailure::StaleModel),
        );
    }
    let Some(current_state) = editable_text_state_for_node(surface, request.node_id) else {
        return rejected(
            &envelope,
            Some(current_model),
            UiNumberModelUpdateFailure::InvalidTarget,
        );
    };
    let Some(value_property) = editable_value_property(surface, request.node_id) else {
        return rejected(
            &envelope,
            Some(current_model),
            UiNumberModelUpdateFailure::InvalidTarget,
        );
    };
    let preserve_edit = request.origin == UiNumberModelUpdateOrigin::BoundRefresh
        && number_field_edit_is_active(surface, request.node_id);
    if preserve_edit {
        crate::profile_counter!("runtime", NUMBER_MODEL_PROFILE_COUNTER_NAMES[1], 1);
    }
    let next_state = if preserve_edit {
        current_state
    } else {
        canonical_state(request.value, current_state.read_only)
    };
    let transaction = match prepare_number_field_model_update_properties(
        surface,
        request.node_id,
        value_property.as_str(),
        UiValue::Float(request.value),
        &next_state,
        preserve_edit,
        preserve_edit,
    )
    .and_then(|prepared| prepared.commit())
    {
        Ok(transaction) => transaction,
        Err(error) => {
            let failure = match error {
                UiEditableTextPropertyTransactionError::NumberRevisionExhausted => {
                    UiNumberModelUpdateFailure::RevisionExhausted
                }
                _ => UiNumberModelUpdateFailure::PropertyRejected,
            };
            return rejected(&envelope, Some(current_model), failure);
        }
    };
    let current_model = match state.model_key(surface, request.node_id) {
        Ok(key) => key,
        Err(failure) => return rejected(&envelope, None, failure),
    };
    let status = if transaction.changed_properties.is_empty() {
        UiNumberModelUpdateStatus::Unchanged
    } else {
        if current_model.revision != envelope.expected_model.revision {
            crate::profile_counter!("runtime", NUMBER_MODEL_PROFILE_COUNTER_NAMES[3], 1);
        }
        UiNumberModelUpdateStatus::Applied
    };
    receipt(&envelope, status, Some(current_model), None)
}

fn canonical_state(value: f64, read_only: bool) -> UiEditableTextState {
    let text = UiValue::Float(value).display_text();
    UiEditableTextState {
        caret: UiTextCaret {
            offset: text.len(),
            affinity: UiTextCaretAffinity::Downstream,
        },
        selection: None,
        composition: None,
        read_only,
        text,
    }
}

fn map_revision_error(error: NumberFieldRevisionError) -> UiNumberModelUpdateFailure {
    match error {
        NumberFieldRevisionError::InvalidState => UiNumberModelUpdateFailure::InvalidTarget,
        NumberFieldRevisionError::Exhausted => UiNumberModelUpdateFailure::RevisionExhausted,
    }
}

fn rejected(
    envelope: &UiNumberModelUpdateEnvelope,
    current_model: Option<UiNumberModelKey>,
    failure: UiNumberModelUpdateFailure,
) -> UiNumberModelUpdateReceipt {
    crate::profile_counter!("runtime", NUMBER_MODEL_PROFILE_COUNTER_NAMES[4], 1);
    receipt(
        envelope,
        UiNumberModelUpdateStatus::Rejected,
        current_model,
        Some(failure),
    )
}

fn receipt(
    envelope: &UiNumberModelUpdateEnvelope,
    status: UiNumberModelUpdateStatus,
    current_model: Option<UiNumberModelKey>,
    failure: Option<UiNumberModelUpdateFailure>,
) -> UiNumberModelUpdateReceipt {
    let receipt = UiNumberModelUpdateReceipt {
        schema_version: UI_NUMBER_MODEL_UPDATE_SCHEMA_VERSION,
        request_id: envelope.request_id,
        tree_id: envelope.tree_id.clone(),
        node_id: envelope.node_id,
        origin: envelope.origin,
        status,
        expected_model: envelope.expected_model,
        current_model,
        failure,
    };
    debug_assert!(receipt.validate().is_ok());
    receipt
}

#[cfg(test)]
#[path = "number_model_updates/tests.rs"]
mod tests;
