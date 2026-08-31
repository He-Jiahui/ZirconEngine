use std::collections::{BTreeMap, VecDeque};

use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiTreeId},
    text::{
        UiTextDocumentKey, UiTextModelUpdateFailure, UiTextModelUpdateId, UiTextModelUpdateOrigin,
        UiTextModelUpdateReceipt, UiTextModelUpdateRequest, UiTextModelUpdateStatus,
    },
};

use crate::ui::{
    dispatch::UiTextDocumentSession,
    surface::{
        UiPendingSecureTextModelUpdateStoreHandle, UiSurface, editable_text_input_is_secure,
        input::{editable_text_state_for_node, editable_value_property, is_editable_text_input},
    },
};

use crate::ui::surface::UiSurfaceSessionIdentityHandle;

use super::{manager::UiInputManager, text_focus_lifecycle::finish_pending_text_focus_loss};

mod profile;
mod transaction;

use transaction::{apply_now, committed_document_state, receipt};

const MVP_MAX_PENDING_TEXT_MODEL_UPDATES: usize = 256;
const MVP_MAX_PENDING_TEXT_MODEL_UPDATE_BYTES: usize = 16 * 1024 * 1024;
const MVP_MAX_TEXT_MODEL_UPDATE_VALUE_BYTES: usize = 4 * 1024 * 1024;

impl UiInputManager {
    pub(crate) fn synchronize_text_document_owners(&mut self, surface: &mut UiSurface) {
        self.text_model_updates.activate_surface(surface);
        self.number_model_updates.activate_surface(surface);
        self.text_documents
            .synchronize_owners(&surface.tree, surface.session_identity());
        self.text_model_updates
            .reconcile_owners(&mut self.text_documents, surface);
        self.number_model_updates.reconcile_owners(surface);
        finish_pending_text_focus_loss(
            &mut self.text_documents,
            &mut self.text_model_updates,
            surface,
        );
    }

    /// Returns the manager-owned document identity currently backing an editable text owner.
    pub fn text_document_key(
        &mut self,
        surface: &mut UiSurface,
        owner: UiNodeId,
    ) -> Result<UiTextDocumentKey, UiTextModelUpdateFailure> {
        self.synchronize_text_document_owners(surface);
        current_document_key(&mut self.text_documents, surface, owner)
    }

    /// Applies or defers one revision-qualified model update without overwriting focused edits.
    pub fn update_text_model(
        &mut self,
        surface: &mut UiSurface,
        request: UiTextModelUpdateRequest,
    ) -> UiTextModelUpdateReceipt {
        crate::profile_scope!("runtime", "ui_text.model_update", "apply_request");
        profile::record_request(
            request.value.len(),
            request.origin,
            surface.focus.focused == Some(request.node_id),
        );
        self.synchronize_text_document_owners(surface);
        let receipt = apply_request(
            &mut self.text_model_updates,
            &mut self.text_documents,
            surface,
            request,
        );
        profile::record_receipt(receipt.status);
        receipt
    }

    /// Drains terminal receipts produced after deferred updates leave the focused state.
    pub fn drain_text_model_update_receipts(&mut self) -> Vec<UiTextModelUpdateReceipt> {
        self.text_model_updates.drain_receipts()
    }
}

#[derive(Clone)]
struct UiTextModelUpdateEnvelope {
    request_id: UiTextModelUpdateId,
    tree_id: UiTreeId,
    node_id: UiNodeId,
    expected_document: UiTextDocumentKey,
    origin: UiTextModelUpdateOrigin,
}

impl From<&UiTextModelUpdateRequest> for UiTextModelUpdateEnvelope {
    fn from(request: &UiTextModelUpdateRequest) -> Self {
        Self {
            request_id: request.request_id,
            tree_id: request.tree_id.clone(),
            node_id: request.node_id,
            expected_document: request.expected_document,
            origin: request.origin,
        }
    }
}

struct UiPendingTextModelUpdate {
    envelope: UiTextModelUpdateEnvelope,
    byte_len: usize,
    secure: bool,
    plain_value: Option<String>,
}

#[derive(Default)]
pub(super) struct UiTextModelUpdateState {
    active_surface: Option<UiSurfaceSessionIdentityHandle>,
    active_secure_store: Option<UiPendingSecureTextModelUpdateStoreHandle>,
    pending: BTreeMap<UiNodeId, UiPendingTextModelUpdate>,
    pending_bytes: usize,
    terminal_receipts: VecDeque<UiTextModelUpdateReceipt>,
}

impl UiTextModelUpdateState {
    pub(super) fn activate_surface(&mut self, surface: &mut UiSurface) {
        let identity = surface.session_identity();
        if self.active_surface.as_ref() == Some(&identity) {
            return;
        }
        if let Some(previous_store) = self.active_secure_store.take() {
            previous_store.clear();
        }
        self.active_surface = Some(identity);
        let active_secure_store = surface.pending_secure_text_model_update_store_handle();
        active_secure_store.clear();
        self.active_secure_store = Some(active_secure_store);
        let abandoned = std::mem::take(&mut self.pending);
        self.pending_bytes = 0;
        for pending in abandoned.into_values() {
            profile::record_pending_release(pending.byte_len);
            self.push_terminal(receipt(
                &pending.envelope,
                UiTextModelUpdateStatus::Rejected,
                None,
                None,
                Some(UiTextModelUpdateFailure::OwnerDetached),
            ));
        }
    }

    pub(super) fn reconcile_owners(
        &mut self,
        text_documents: &mut UiTextDocumentSession,
        surface: &mut UiSurface,
    ) {
        let rejected = self
            .pending
            .iter()
            .filter_map(|(owner, pending)| {
                if !surface.tree.nodes.contains_key(owner)
                    || !is_editable_text_input(surface, *owner)
                {
                    return Some((*owner, UiTextModelUpdateFailure::OwnerDetached));
                }
                (editable_text_input_is_secure(surface, *owner) != pending.secure)
                    .then_some((*owner, UiTextModelUpdateFailure::SecurityPolicyChanged))
            })
            .collect::<Vec<_>>();
        for (owner, failure) in rejected {
            let current_document = current_document_key(text_documents, surface, owner).ok();
            self.finish_rejected(surface, owner, current_document, failure);
        }
    }

    pub(super) fn finish_focus_loss(
        &mut self,
        text_documents: &mut UiTextDocumentSession,
        surface: &mut UiSurface,
        owner: UiNodeId,
    ) {
        let Some(pending) = self.remove_pending(owner) else {
            return;
        };
        let current_document = match current_document_key(text_documents, surface, owner) {
            Ok(current_document) => current_document,
            Err(failure) => {
                self.discard_secure_pending(surface, owner, pending.secure);
                self.push_terminal(receipt(
                    &pending.envelope,
                    UiTextModelUpdateStatus::Rejected,
                    None,
                    None,
                    Some(failure),
                ));
                return;
            }
        };
        if current_document != pending.envelope.expected_document {
            self.discard_secure_pending(surface, owner, pending.secure);
            self.push_terminal(receipt(
                &pending.envelope,
                UiTextModelUpdateStatus::Conflict,
                Some(current_document),
                None,
                Some(UiTextModelUpdateFailure::StaleDocument),
            ));
            return;
        }
        let value = if pending.secure {
            surface.take_pending_secure_text_model_update(owner)
        } else {
            pending.plain_value
        };
        let Some(value) = value else {
            self.push_terminal(receipt(
                &pending.envelope,
                UiTextModelUpdateStatus::Rejected,
                Some(current_document),
                None,
                Some(UiTextModelUpdateFailure::SecureValueUnavailable),
            ));
            return;
        };
        self.push_terminal(apply_now(text_documents, surface, pending.envelope, value));
    }

    pub(super) fn finish_all_unfocused(
        &mut self,
        text_documents: &mut UiTextDocumentSession,
        surface: &mut UiSurface,
    ) {
        let focused = surface.focus.focused;
        let owners = self
            .pending
            .keys()
            .copied()
            .filter(|owner| Some(*owner) != focused)
            .collect::<Vec<_>>();
        for owner in owners {
            self.finish_focus_loss(text_documents, surface, owner);
        }
    }

    fn defer(
        &mut self,
        surface: &mut UiSurface,
        request: UiTextModelUpdateRequest,
        current_document: UiTextDocumentKey,
        secure: bool,
    ) -> UiTextModelUpdateReceipt {
        let envelope = UiTextModelUpdateEnvelope::from(&request);
        let (existing_bytes, supersedes_existing) = self
            .pending
            .get(&request.node_id)
            .map_or((0, false), |pending| (pending.byte_len, true));
        let occupied_receipts = self
            .terminal_receipts
            .len()
            .saturating_add(self.pending.len());
        if occupied_receipts >= MVP_MAX_PENDING_TEXT_MODEL_UPDATES {
            return receipt(
                &envelope,
                UiTextModelUpdateStatus::Rejected,
                Some(current_document),
                None,
                Some(UiTextModelUpdateFailure::PendingQueueFull),
            );
        }
        if request.value.len() > MVP_MAX_TEXT_MODEL_UPDATE_VALUE_BYTES {
            return receipt(
                &envelope,
                UiTextModelUpdateStatus::Rejected,
                Some(current_document),
                None,
                Some(UiTextModelUpdateFailure::ValueTooLarge),
            );
        }
        let Some(next_pending_bytes) = self
            .pending_bytes
            .checked_sub(existing_bytes)
            .and_then(|bytes| bytes.checked_add(request.value.len()))
        else {
            return receipt(
                &envelope,
                UiTextModelUpdateStatus::Rejected,
                Some(current_document),
                None,
                Some(UiTextModelUpdateFailure::PendingBytesExceeded),
            );
        };
        if next_pending_bytes > MVP_MAX_PENDING_TEXT_MODEL_UPDATE_BYTES {
            return receipt(
                &envelope,
                UiTextModelUpdateStatus::Rejected,
                Some(current_document),
                None,
                Some(UiTextModelUpdateFailure::PendingBytesExceeded),
            );
        }

        if let Some(previous) = self.pending.remove(&request.node_id) {
            self.discard_secure_pending(surface, request.node_id, previous.secure);
            self.push_terminal(receipt(
                &previous.envelope,
                UiTextModelUpdateStatus::Rejected,
                Some(current_document),
                None,
                Some(UiTextModelUpdateFailure::Superseded),
            ));
        }
        let byte_len = request.value.len();
        let plain_value = if secure {
            surface.store_pending_secure_text_model_update(request.node_id, request.value);
            None
        } else {
            Some(request.value)
        };
        self.pending.insert(
            request.node_id,
            UiPendingTextModelUpdate {
                envelope: envelope.clone(),
                byte_len,
                secure,
                plain_value,
            },
        );
        self.pending_bytes = next_pending_bytes;
        profile::record_pending_admission(byte_len, supersedes_existing);
        receipt(
            &envelope,
            UiTextModelUpdateStatus::Deferred,
            Some(current_document),
            None,
            None,
        )
    }

    fn supersede_pending(
        &mut self,
        surface: &mut UiSurface,
        owner: UiNodeId,
        current_document: Option<UiTextDocumentKey>,
    ) {
        let Some(pending) = self.remove_pending(owner) else {
            return;
        };
        self.discard_secure_pending(surface, owner, pending.secure);
        self.push_terminal(receipt(
            &pending.envelope,
            UiTextModelUpdateStatus::Rejected,
            current_document,
            None,
            Some(UiTextModelUpdateFailure::Superseded),
        ));
    }

    fn finish_rejected(
        &mut self,
        surface: &mut UiSurface,
        owner: UiNodeId,
        current_document: Option<UiTextDocumentKey>,
        failure: UiTextModelUpdateFailure,
    ) {
        let Some(pending) = self.remove_pending(owner) else {
            return;
        };
        self.discard_secure_pending(surface, owner, pending.secure);
        self.push_terminal(receipt(
            &pending.envelope,
            UiTextModelUpdateStatus::Rejected,
            current_document,
            None,
            Some(failure),
        ));
    }

    fn remove_pending(&mut self, owner: UiNodeId) -> Option<UiPendingTextModelUpdate> {
        let pending = self.pending.remove(&owner)?;
        self.pending_bytes = self.pending_bytes.saturating_sub(pending.byte_len);
        profile::record_pending_release(pending.byte_len);
        Some(pending)
    }

    fn discard_secure_pending(&self, surface: &mut UiSurface, owner: UiNodeId, secure: bool) {
        if secure {
            surface.discard_pending_secure_text_model_update(owner);
        }
    }

    fn push_terminal(&mut self, receipt: UiTextModelUpdateReceipt) {
        debug_assert!(self.terminal_receipts.len() < MVP_MAX_PENDING_TEXT_MODEL_UPDATES);
        profile::record_receipt(receipt.status);
        self.terminal_receipts.push_back(receipt);
    }

    pub(super) fn drain_receipts(&mut self) -> Vec<UiTextModelUpdateReceipt> {
        self.terminal_receipts.drain(..).collect()
    }
}

impl Drop for UiTextModelUpdateState {
    fn drop(&mut self) {
        if let Some(active_secure_store) = &self.active_secure_store {
            active_secure_store.clear();
        }
    }
}

pub(super) fn current_document_key(
    text_documents: &mut UiTextDocumentSession,
    surface: &UiSurface,
    owner: UiNodeId,
) -> Result<UiTextDocumentKey, UiTextModelUpdateFailure> {
    if !surface.tree.nodes.contains_key(&owner) || !is_editable_text_input(surface, owner) {
        return Err(UiTextModelUpdateFailure::InvalidTarget);
    }
    let state = editable_text_state_for_node(surface, owner)
        .ok_or(UiTextModelUpdateFailure::InvalidTarget)?;
    let value_property =
        editable_value_property(surface, owner).ok_or(UiTextModelUpdateFailure::InvalidTarget)?;
    let string_value = surface
        .tree
        .node(owner)
        .and_then(|node| node.template_metadata.as_ref())
        .and_then(|metadata| metadata.attributes.get(&value_property))
        .is_some_and(toml::Value::is_str);
    if !string_value {
        return Err(UiTextModelUpdateFailure::InvalidTarget);
    }
    let source_epoch = surface
        .input
        .text_document_epoch(owner)
        .ok_or(UiTextModelUpdateFailure::DocumentUnavailable)?;
    let committed_state = committed_document_state(state);
    text_documents.synchronize_source(
        &surface.tree.tree_id,
        owner,
        source_epoch,
        committed_state.text.as_str(),
    );
    text_documents
        .document_key(&surface.tree.tree_id, owner, source_epoch)
        .map_err(|_| UiTextModelUpdateFailure::DocumentUnavailable)
}

pub(super) fn apply_request(
    state: &mut UiTextModelUpdateState,
    text_documents: &mut UiTextDocumentSession,
    surface: &mut UiSurface,
    request: UiTextModelUpdateRequest,
) -> UiTextModelUpdateReceipt {
    let envelope = UiTextModelUpdateEnvelope::from(&request);
    if let Err(failure) = request.validate() {
        return receipt(
            &envelope,
            UiTextModelUpdateStatus::Rejected,
            None,
            None,
            Some(failure),
        );
    }
    if request.value.len() > MVP_MAX_TEXT_MODEL_UPDATE_VALUE_BYTES {
        return receipt(
            &envelope,
            UiTextModelUpdateStatus::Rejected,
            None,
            None,
            Some(UiTextModelUpdateFailure::ValueTooLarge),
        );
    }
    if request.tree_id != surface.tree.tree_id {
        return receipt(
            &envelope,
            UiTextModelUpdateStatus::Rejected,
            None,
            None,
            Some(UiTextModelUpdateFailure::WrongTree),
        );
    }
    let current_document = match current_document_key(text_documents, surface, request.node_id) {
        Ok(current_document) => current_document,
        Err(failure) => {
            return receipt(
                &envelope,
                UiTextModelUpdateStatus::Rejected,
                None,
                None,
                Some(failure),
            );
        }
    };
    if request.expected_document != current_document {
        return receipt(
            &envelope,
            UiTextModelUpdateStatus::Conflict,
            Some(current_document),
            None,
            Some(UiTextModelUpdateFailure::StaleDocument),
        );
    }
    let secure = editable_text_input_is_secure(surface, request.node_id);
    profile::record_security_class(secure);
    if request.origin == UiTextModelUpdateOrigin::BoundRefresh
        && editable_text_state_for_node(surface, request.node_id).is_some_and(|editable| {
            editable.composition.is_none() && editable.text == request.value
        })
    {
        state.supersede_pending(surface, request.node_id, Some(current_document));
        return receipt(
            &envelope,
            UiTextModelUpdateStatus::Unchanged,
            Some(current_document),
            None,
            None,
        );
    }
    if request.origin == UiTextModelUpdateOrigin::BoundRefresh
        && surface.focus.focused == Some(request.node_id)
    {
        return state.defer(surface, request, current_document, secure);
    }

    let owner = request.node_id;
    let result = apply_now(text_documents, surface, envelope, request.value);
    if matches!(
        result.status,
        UiTextModelUpdateStatus::Applied | UiTextModelUpdateStatus::Unchanged
    ) {
        state.supersede_pending(surface, owner, result.current_document);
    }
    result
}

#[cfg(test)]
#[path = "bound_text_model_updates/tests.rs"]
mod tests;
