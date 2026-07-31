use std::collections::HashMap;
use std::sync::Arc;

use super::super::{
    EditorEventRetentionBudgets, EditorEventRetentionPolicy, EditorEventRetentionStore,
    SharedEditorEventRecord,
};
use super::{
    EditorEventListenerDelivery, EditorEventListenerDescriptor, EditorEventListenerFilter,
    EditorEventListenerStatus,
};

#[derive(Debug)]
struct EditorEventListenerState {
    descriptor: EditorEventListenerDescriptor,
    inbox: EditorEventRetentionStore,
}

#[derive(Debug)]
pub struct EditorEventListenerRegistry {
    listener_order: Vec<String>,
    listeners: HashMap<String, EditorEventListenerState>,
    retention_budgets: EditorEventRetentionBudgets,
}

impl Default for EditorEventListenerRegistry {
    fn default() -> Self {
        Self::new(EditorEventRetentionPolicy::default().listeners)
    }
}

impl EditorEventListenerRegistry {
    pub(crate) fn new(retention_budgets: EditorEventRetentionBudgets) -> Self {
        Self {
            listener_order: Vec::new(),
            listeners: HashMap::new(),
            retention_budgets,
        }
    }

    pub fn register(
        &mut self,
        listener_id: impl Into<String>,
        display_name: impl Into<String>,
    ) -> Result<(), String> {
        let listener_id = listener_id.into();
        if self.listeners.contains_key(&listener_id) {
            return Err(format!(
                "editor event listener {listener_id} already registered"
            ));
        }
        self.listener_order.push(listener_id.clone());
        self.listeners.insert(
            listener_id.clone(),
            EditorEventListenerState {
                descriptor: EditorEventListenerDescriptor {
                    listener_id,
                    display_name: display_name.into(),
                    enabled: true,
                    filter: None,
                },
                inbox: EditorEventRetentionStore::new(self.retention_budgets.clone()),
            },
        );
        Ok(())
    }

    pub fn unregister(&mut self, listener_id: &str) -> Result<(), String> {
        if self.listeners.remove(listener_id).is_none() {
            return Err(not_registered(listener_id));
        }
        self.listener_order.retain(|id| id != listener_id);
        Ok(())
    }

    pub fn set_enabled(&mut self, listener_id: &str, enabled: bool) -> Result<(), String> {
        self.listener_mut(listener_id)?.descriptor.enabled = enabled;
        Ok(())
    }

    pub fn set_filter(
        &mut self,
        listener_id: &str,
        filter: EditorEventListenerFilter,
    ) -> Result<(), String> {
        self.listener_mut(listener_id)?.descriptor.filter = Some(filter.normalized());
        Ok(())
    }

    pub fn clear_filter(&mut self, listener_id: &str) -> Result<(), String> {
        self.listener_mut(listener_id)?.descriptor.filter = None;
        Ok(())
    }

    pub fn listeners(&self) -> Vec<EditorEventListenerDescriptor> {
        self.listener_order
            .iter()
            .filter_map(|listener_id| self.listeners.get(listener_id))
            .map(|listener| listener.descriptor.clone())
            .collect()
    }

    pub fn status_for(&mut self, listener_id: &str) -> Result<EditorEventListenerStatus, String> {
        let listener = self.listener_mut(listener_id)?;
        let retention = listener.inbox.diagnostics();
        let first_pending_sequence = retention.first_retained_sequence();
        let last_pending_sequence = retention.last_retained_sequence();
        let retention_budgets = listener.inbox.budgets();
        Ok(EditorEventListenerStatus {
            listener_id: listener.descriptor.listener_id.clone(),
            descriptor: listener.descriptor.clone(),
            pending_delivery_count: retention.retained_records(),
            pending_delivery_bytes: retention.retained_bytes(),
            first_pending_sequence,
            last_pending_sequence,
            dropped_delivery_count: retention.dropped_records(),
            coalesced_delivery_count: retention.coalesced_records(),
            lagged_since_sequence: retention.first_dropped_sequence(),
            last_dropped_sequence: retention.last_dropped_sequence(),
            retention_budgets,
            retention,
        })
    }

    pub fn deliveries_for(
        &mut self,
        listener_id: &str,
    ) -> Result<Vec<EditorEventListenerDelivery>, String> {
        self.deliveries_after(listener_id, None)
    }

    pub fn deliveries_after_sequence(
        &mut self,
        listener_id: &str,
        after_sequence: u64,
    ) -> Result<Vec<EditorEventListenerDelivery>, String> {
        self.deliveries_after(listener_id, Some(after_sequence))
    }

    pub fn acknowledge_through(
        &mut self,
        listener_id: &str,
        sequence: u64,
    ) -> Result<usize, String> {
        Ok(self
            .listener_mut(listener_id)?
            .inbox
            .acknowledge_through(sequence))
    }

    pub(crate) fn notify(&mut self, record: Arc<SharedEditorEventRecord>) {
        for listener_id in &self.listener_order {
            let Some(listener) = self.listeners.get_mut(listener_id) else {
                continue;
            };
            if !listener.descriptor.enabled
                || listener
                    .descriptor
                    .filter
                    .as_ref()
                    .is_some_and(|filter| !filter.accepts(record.record()))
            {
                continue;
            }
            listener.inbox.push(Arc::clone(&record));
        }
    }

    fn deliveries_after(
        &mut self,
        listener_id: &str,
        after_sequence: Option<u64>,
    ) -> Result<Vec<EditorEventListenerDelivery>, String> {
        let listener = self.listener_mut(listener_id)?;
        Ok(listener
            .inbox
            .records()
            .into_iter()
            .filter(|payload| {
                after_sequence.is_none_or(|sequence| payload.record().sequence.0 > sequence)
            })
            .map(|payload| EditorEventListenerDelivery::from_shared(listener_id, payload.as_ref()))
            .collect())
    }

    fn listener_mut(&mut self, listener_id: &str) -> Result<&mut EditorEventListenerState, String> {
        self.listeners
            .get_mut(listener_id)
            .ok_or_else(|| not_registered(listener_id))
    }
}

fn not_registered(listener_id: &str) -> String {
    format!("editor event listener {listener_id} is not registered")
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use serde_json::json;

    use super::super::super::{
        EditorEvent, EditorEventEffect, EditorEventId, EditorEventRecord, EditorEventResult,
        EditorEventSequence, EditorEventSource, EditorEventTransient, EditorEventUndoPolicy,
        SharedEditorEventRecord,
    };
    use super::EditorEventListenerRegistry;

    #[test]
    fn matching_listener_inboxes_share_one_immutable_payload() {
        let mut listeners = EditorEventListenerRegistry::default();
        listeners.register("a", "A").unwrap();
        listeners.register("b", "B").unwrap();
        let payload = Arc::new(SharedEditorEventRecord::new(EditorEventRecord {
            event_id: EditorEventId::new(1),
            sequence: EditorEventSequence::new(1),
            source: EditorEventSource::Headless,
            event: EditorEvent::Transient(EditorEventTransient::OpenCommandPalette),
            operation_id: None,
            operation_display_name: None,
            operation_arguments: None,
            operation_group: None,
            effects: Vec::<EditorEventEffect>::new(),
            undo_policy: EditorEventUndoPolicy::NonUndoable,
            before_revision: 0,
            after_revision: 1,
            result: EditorEventResult::success(json!(null)),
        }));

        listeners.notify(Arc::clone(&payload));

        assert_eq!(Arc::strong_count(&payload), 3);
    }
}
