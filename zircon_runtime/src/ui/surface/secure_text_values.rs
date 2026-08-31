use std::{
    collections::BTreeMap,
    fmt,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;
use zircon_runtime_interface::ui::{
    component::{UiComponentEvent, UiSecureTextValueRef, UiValue},
    dispatch::{UiDispatchEffect, UiDispatchHostRequestKind, UiInputDispatchResult, UiInputEvent},
    event_ui::UiNodeId,
};

use super::{UiSurface, input::editable_value_property};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UiTextComponentEventKind {
    Change,
    Commit,
}

#[derive(Clone, Default)]
pub(in crate::ui) struct UiPendingSecureTextModelUpdateStoreHandle {
    values: Arc<Mutex<BTreeMap<UiNodeId, UiPendingSecureTextModelValue>>>,
}

struct UiPendingSecureTextModelValue {
    value: Zeroizing<String>,
}

impl UiPendingSecureTextModelValue {
    fn new(value: String) -> Self {
        Self {
            value: Zeroizing::new(value),
        }
    }

    fn len(&self) -> usize {
        self.value.len()
    }

    fn into_string(mut self) -> String {
        std::mem::take(&mut *self.value)
    }
}

impl fmt::Debug for UiPendingSecureTextModelUpdateStoreHandle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (count, bytes) = self.metrics();
        formatter
            .debug_struct("UiPendingSecureTextModelUpdateStoreHandle")
            .field("count", &count)
            .field("bytes", &bytes)
            .finish()
    }
}

impl UiPendingSecureTextModelUpdateStoreHandle {
    fn with_values<Result>(
        &self,
        action: impl FnOnce(&mut BTreeMap<UiNodeId, UiPendingSecureTextModelValue>) -> Result,
    ) -> Result {
        let mut values = match self.values.lock() {
            Ok(values) => values,
            Err(poisoned) => poisoned.into_inner(),
        };
        action(&mut values)
    }

    fn store(&self, owner: UiNodeId, value: String) {
        self.with_values(|values| {
            values.insert(owner, UiPendingSecureTextModelValue::new(value));
        });
    }

    fn take(&self, owner: UiNodeId) -> Option<String> {
        self.with_values(|values| {
            values
                .remove(&owner)
                .map(UiPendingSecureTextModelValue::into_string)
        })
    }

    fn discard(&self, owner: UiNodeId) -> bool {
        self.with_values(|values| values.remove(&owner).is_some())
    }

    pub(in crate::ui) fn clear(&self) {
        self.with_values(BTreeMap::clear);
    }

    fn metrics(&self) -> (usize, usize) {
        self.with_values(|values| {
            (
                values.len(),
                values
                    .values()
                    .map(UiPendingSecureTextModelValue::len)
                    .sum::<usize>(),
            )
        })
    }
}

#[derive(Default, Serialize, Deserialize)]
pub(super) struct UiSurfaceSecureTextValueStore {
    #[serde(skip)]
    current: BTreeMap<UiNodeId, BTreeMap<String, UiSecureTextValueLease>>,
    #[serde(skip)]
    pending_model_updates: UiPendingSecureTextModelUpdateStoreHandle,
}

impl fmt::Debug for UiSurfaceSecureTextValueStore {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lease_count = self.current.values().map(BTreeMap::len).sum::<usize>();
        let (pending_model_update_count, pending_model_update_bytes) =
            self.pending_model_updates.metrics();
        formatter
            .debug_struct("UiSurfaceSecureTextValueStore")
            .field("lease_count", &lease_count)
            .field("pending_model_update_count", &pending_model_update_count)
            .field("pending_model_update_bytes", &pending_model_update_bytes)
            .finish()
    }
}

impl Clone for UiSurfaceSecureTextValueStore {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl PartialEq for UiSurfaceSecureTextValueStore {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[derive(Clone, Debug, PartialEq)]
struct UiSecureTextValueLease {
    reference: UiSecureTextValueRef,
    text_layout_revision: Option<u64>,
}

impl UiSurfaceSecureTextValueStore {
    fn register(&mut self, reference: UiSecureTextValueRef, text_layout_revision: Option<u64>) {
        self.current.entry(reference.node_id()).or_default().insert(
            reference.property().to_string(),
            UiSecureTextValueLease {
                reference,
                text_layout_revision,
            },
        );
    }

    fn resolves(
        &self,
        reference: &UiSecureTextValueRef,
        text_layout_revision: Option<u64>,
    ) -> bool {
        let Some(revision) = text_layout_revision else {
            return false;
        };
        self.current
            .get(&reference.node_id())
            .and_then(|properties| properties.get(reference.property()))
            .is_some_and(|lease| {
                lease.reference == *reference && lease.text_layout_revision == Some(revision)
            })
    }

    fn revoke(&mut self, reference: &UiSecureTextValueRef) -> bool {
        let Some(properties) = self.current.get_mut(&reference.node_id()) else {
            return false;
        };
        let matches_current = properties
            .get(reference.property())
            .is_some_and(|lease| lease.reference == *reference);
        if !matches_current {
            return false;
        }
        properties.remove(reference.property());
        if properties.is_empty() {
            self.current.remove(&reference.node_id());
        }
        true
    }

    fn store_pending_model_update(&mut self, owner: UiNodeId, value: String) {
        self.pending_model_updates.store(owner, value);
    }

    fn take_pending_model_update(&mut self, owner: UiNodeId) -> Option<String> {
        self.pending_model_updates.take(owner)
    }

    fn discard_pending_model_update(&mut self, owner: UiNodeId) -> bool {
        self.pending_model_updates.discard(owner)
    }
}

impl UiSurface {
    pub(crate) fn component_value_event(
        &mut self,
        target: UiNodeId,
        property: String,
        value: UiValue,
        kind: UiTextComponentEventKind,
    ) -> UiComponentEvent {
        if let UiValue::String(value) = value {
            return self.text_component_event(target, property, value, kind);
        }
        match kind {
            UiTextComponentEventKind::Change => UiComponentEvent::ValueChanged { property, value },
            UiTextComponentEventKind::Commit => UiComponentEvent::Commit { property, value },
        }
    }

    pub(crate) fn text_component_event(
        &mut self,
        target: UiNodeId,
        property: String,
        value: String,
        kind: UiTextComponentEventKind,
    ) -> UiComponentEvent {
        if !super::editable_text_input_is_secure(self, target) {
            return match kind {
                UiTextComponentEventKind::Change => UiComponentEvent::ValueChanged {
                    property,
                    value: UiValue::String(value),
                },
                UiTextComponentEventKind::Commit => UiComponentEvent::Commit {
                    property,
                    value: UiValue::String(value),
                },
            };
        }

        let reference = UiSecureTextValueRef::issue(self.tree.tree_id.clone(), target, &property);
        let text_layout_revision = self
            .tree
            .node(target)
            .and_then(|node| node.layout_cache.retained_text_layout_revision());
        self.secure_text_values
            .register(reference.clone(), text_layout_revision);
        match kind {
            UiTextComponentEventKind::Change => UiComponentEvent::SecureValueChanged {
                property,
                reference,
            },
            UiTextComponentEventKind::Commit => UiComponentEvent::SecureCommit {
                property,
                reference,
            },
        }
    }

    /// Resolves the latest secure text event reference against this exact surface.
    ///
    /// Old references stop resolving after the field changes, and references are rejected when the
    /// target is no longer a secure editable-text node or the property identity no longer matches.
    pub(crate) fn resolve_secure_text_value<'a>(
        &'a self,
        reference: &UiSecureTextValueRef,
    ) -> Option<&'a str> {
        if reference.tree_id() != &self.tree.tree_id
            || !super::editable_text_input_is_secure(self, reference.node_id())
            || editable_value_property(self, reference.node_id()).as_deref()
                != Some(reference.property())
        {
            return None;
        }
        let text_layout_revision = self
            .tree
            .node(reference.node_id())
            .and_then(|node| node.layout_cache.retained_text_layout_revision());
        if !self
            .secure_text_values
            .resolves(reference, text_layout_revision)
        {
            return None;
        }

        self.component_states
            .get(reference.node_id())
            .and_then(|state| state.value(reference.property()))
            .and_then(string_value)
            .or_else(|| {
                self.tree
                    .node(reference.node_id())
                    .and_then(|node| node.template_metadata.as_ref())
                    .and_then(|metadata| metadata.attributes.get(reference.property()))
                    .and_then(toml::Value::as_str)
            })
    }

    pub(crate) fn revoke_secure_text_value(&mut self, reference: &UiSecureTextValueRef) -> bool {
        self.secure_text_values.revoke(reference)
    }

    pub(crate) fn store_pending_secure_text_model_update(
        &mut self,
        owner: UiNodeId,
        value: String,
    ) {
        self.secure_text_values
            .store_pending_model_update(owner, value);
    }

    pub(crate) fn take_pending_secure_text_model_update(
        &mut self,
        owner: UiNodeId,
    ) -> Option<String> {
        self.secure_text_values.take_pending_model_update(owner)
    }

    pub(crate) fn discard_pending_secure_text_model_update(&mut self, owner: UiNodeId) -> bool {
        self.secure_text_values.discard_pending_model_update(owner)
    }

    pub(in crate::ui) fn pending_secure_text_model_update_store_handle(
        &self,
    ) -> UiPendingSecureTextModelUpdateStoreHandle {
        self.secure_text_values.pending_model_updates.clone()
    }

    pub(crate) fn redact_secure_text_result(
        &mut self,
        target: UiNodeId,
        result: &mut UiInputDispatchResult,
    ) {
        if !super::editable_text_input_is_secure(self, target) {
            return;
        }
        result.diagnostics.secure_text_redacted = true;
        redact_input_event(&mut result.event);
        for report in &mut result.binding_reports {
            for update in &mut report.updates {
                update.previous = None;
                update.value = UiValue::Null;
                update.message = None;
            }
        }
        for effect in &mut result.reply.effects {
            redact_dispatch_effect(effect);
        }
        for effect in &mut result.applied_effects {
            redact_dispatch_effect(&mut effect.effect);
        }
        for effect in &mut result.rejected_effects {
            redact_dispatch_effect(&mut effect.effect);
        }
        for request in &mut result.host_requests {
            redact_host_request(&mut request.request);
        }
        for report in &mut result.component_events {
            redact_component_event(&mut report.event);
            if let Some(action) = &mut report.template_action {
                action
                    .payload
                    .values_mut()
                    .for_each(|value| *value = UiValue::Null);
            }
        }
    }

    pub(crate) fn redact_secure_text_dispatch_result(
        &mut self,
        result: &mut UiInputDispatchResult,
    ) {
        let routed_target = match &result.event {
            UiInputEvent::Accessibility(event) => Some(event.request.target),
            UiInputEvent::Clipboard(event) => Some(event.owner),
            UiInputEvent::Keyboard(_) | UiInputEvent::Text(_) | UiInputEvent::Ime(_) => {
                result.diagnostics.route_target.or(self.focus.focused)
            }
            _ => None,
        };
        let target = routed_target
            .filter(|target| super::editable_text_input_is_secure(self, *target))
            .or_else(|| {
                result
                    .component_events
                    .iter()
                    .map(|report| report.target)
                    .find(|target| super::editable_text_input_is_secure(self, *target))
            })
            .or_else(|| {
                result
                    .reply
                    .effects
                    .iter()
                    .filter_map(text_service_effect_owner)
                    .find(|target| super::editable_text_input_is_secure(self, *target))
            });
        if let Some(target) = target {
            self.redact_secure_text_result(target, result);
        }
    }
}

fn string_value(value: &UiValue) -> Option<&str> {
    match value {
        UiValue::String(value) => Some(value),
        _ => None,
    }
}

fn redact_input_event(event: &mut UiInputEvent) {
    match event {
        UiInputEvent::Keyboard(event) => {
            event.physical_key = "Redacted".to_string();
            event.logical_key = "Redacted".to_string();
            event.text = event.text.as_ref().map(|_| String::new());
        }
        UiInputEvent::Text(event) => event.text.clear(),
        UiInputEvent::Ime(event) => {
            event.text.clear();
            event.cursor_range = None;
            event.preedit_clauses.clear();
        }
        UiInputEvent::Clipboard(event) => {
            if let zircon_runtime_interface::ui::dispatch::UiClipboardTransferOutcome::ReadText {
                text,
            } = &mut event.outcome
            {
                text.clear();
            }
        }
        UiInputEvent::Accessibility(event) => {
            event.request.value = None;
            event.request.numeric_value = None;
            event.request.text_selection = None;
        }
        _ => {}
    }
}

fn redact_dispatch_effect(effect: &mut UiDispatchEffect) {
    match effect {
        UiDispatchEffect::RequestInputMethod { request } => request.surrounding_text = None,
        UiDispatchEffect::RequestClipboard { request } => request.text = None,
        UiDispatchEffect::EmitComponentEvent { event, .. } => redact_component_event(event),
        _ => {}
    }
}

fn text_service_effect_owner(effect: &UiDispatchEffect) -> Option<UiNodeId> {
    match effect {
        UiDispatchEffect::RequestInputMethod { request } => Some(request.owner),
        UiDispatchEffect::RequestClipboard { request } => Some(request.owner),
        UiDispatchEffect::EmitComponentEvent { target, .. } => Some(*target),
        _ => None,
    }
}

fn redact_host_request(request: &mut UiDispatchHostRequestKind) {
    match request {
        UiDispatchHostRequestKind::InputMethod(request) => request.surrounding_text = None,
        UiDispatchHostRequestKind::Clipboard(request) => request.text = None,
        _ => {}
    }
}

fn redact_component_event(event: &mut UiComponentEvent) {
    match event {
        UiComponentEvent::ValueChanged { value, .. }
        | UiComponentEvent::Commit { value, .. }
        | UiComponentEvent::AddElement { value, .. }
        | UiComponentEvent::SetElement { value, .. }
        | UiComponentEvent::AddMapEntry { value, .. }
        | UiComponentEvent::SetMapEntry { value, .. } => *value = UiValue::Null,
        UiComponentEvent::KeyboardText { text } => text.clear(),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiTreeId};

    use super::*;

    #[test]
    fn revoke_removes_only_the_matching_current_secure_text_lease() {
        let node = UiNodeId::new(7);
        let current = UiSecureTextValueRef::issue(UiTreeId::new("secure.revoke"), node, "value");
        let stale = UiSecureTextValueRef::issue(UiTreeId::new("secure.revoke"), node, "value");
        let mut store = UiSurfaceSecureTextValueStore::default();
        store.register(current.clone(), Some(11));

        assert!(!store.revoke(&stale));
        assert!(store.resolves(&current, Some(11)));
        assert!(store.revoke(&current));
        assert!(!store.resolves(&current, Some(11)));
        assert!(!store.revoke(&current));
    }

    #[test]
    fn pending_model_text_is_surface_owned_and_debug_redacted() {
        let owner = UiNodeId::new(9);
        let mut store = UiSurfaceSecureTextValueStore::default();
        store.store_pending_model_update(owner, "pending-secret-model".to_string());

        let debug = format!("{store:?}");
        assert!(!debug.contains("pending-secret-model"));
        assert!(debug.contains("pending_model_update_count: 1"));
        assert_eq!(
            store.take_pending_model_update(owner).as_deref(),
            Some("pending-secret-model")
        );
        assert!(store.take_pending_model_update(owner).is_none());
    }

    #[test]
    fn accepted_pending_model_text_moves_out_of_the_zeroizing_owner() {
        let owner = UiNodeId::new(10);
        let mut store = UiSurfaceSecureTextValueStore::default();
        store.store_pending_model_update(owner, "accepted-secret-model".to_string());

        let value = store
            .take_pending_model_update(owner)
            .expect("pending secure model value");

        assert_eq!(value, "accepted-secret-model");
        assert!(store.take_pending_model_update(owner).is_none());
    }
}
