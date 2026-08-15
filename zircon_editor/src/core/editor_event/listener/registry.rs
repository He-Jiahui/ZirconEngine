use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::super::{
    EditorEventRetentionBudgets, EditorEventRetentionPolicy, EditorEventRetentionStore,
};
use super::{
    EditorEventListenerDescriptor, EditorEventListenerFilter, EditorEventListenerHandle,
    EditorEventListenerRoute,
};

#[derive(Debug)]
struct EditorEventListenerState {
    descriptor: EditorEventListenerDescriptor,
    inbox: Arc<Mutex<EditorEventRetentionStore>>,
}

#[derive(Debug)]
pub struct EditorEventListenerRegistry {
    listener_order: Vec<String>,
    listeners: HashMap<String, EditorEventListenerState>,
    retention_budgets: EditorEventRetentionBudgets,
    delivery_routes: Arc<[EditorEventListenerRoute]>,
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
            delivery_routes: Arc::from([]),
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
                inbox: Arc::new(Mutex::new(EditorEventRetentionStore::new(
                    self.retention_budgets.clone(),
                ))),
            },
        );
        self.rebuild_delivery_routes();
        Ok(())
    }

    pub fn unregister(&mut self, listener_id: &str) -> Result<(), String> {
        if self.listeners.remove(listener_id).is_none() {
            return Err(not_registered(listener_id));
        }
        self.listener_order.retain(|id| id != listener_id);
        self.rebuild_delivery_routes();
        Ok(())
    }

    pub fn set_enabled(&mut self, listener_id: &str, enabled: bool) -> Result<(), String> {
        self.listener_mut(listener_id)?.descriptor.enabled = enabled;
        self.rebuild_delivery_routes();
        Ok(())
    }

    pub fn set_filter(
        &mut self,
        listener_id: &str,
        filter: EditorEventListenerFilter,
    ) -> Result<(), String> {
        self.listener_mut(listener_id)?.descriptor.filter = Some(filter.normalized());
        self.rebuild_delivery_routes();
        Ok(())
    }

    pub fn clear_filter(&mut self, listener_id: &str) -> Result<(), String> {
        self.listener_mut(listener_id)?.descriptor.filter = None;
        self.rebuild_delivery_routes();
        Ok(())
    }

    pub fn listeners(&self) -> Vec<EditorEventListenerDescriptor> {
        self.listener_order
            .iter()
            .filter_map(|listener_id| self.listeners.get(listener_id))
            .map(|listener| listener.descriptor.clone())
            .collect()
    }

    pub(crate) fn listener_handle(
        &self,
        listener_id: &str,
    ) -> Result<EditorEventListenerHandle, String> {
        let listener = self
            .listeners
            .get(listener_id)
            .ok_or_else(|| not_registered(listener_id))?;
        Ok(EditorEventListenerHandle::new(
            listener.descriptor.clone(),
            Arc::clone(&listener.inbox),
        ))
    }

    pub(crate) fn delivery_routes(&self) -> Arc<[EditorEventListenerRoute]> {
        Arc::clone(&self.delivery_routes)
    }

    fn listener_mut(&mut self, listener_id: &str) -> Result<&mut EditorEventListenerState, String> {
        self.listeners
            .get_mut(listener_id)
            .ok_or_else(|| not_registered(listener_id))
    }

    fn rebuild_delivery_routes(&mut self) {
        self.delivery_routes = self
            .listener_order
            .iter()
            .filter_map(|listener_id| self.listeners.get(listener_id))
            .filter(|listener| listener.descriptor.enabled)
            .map(|listener| {
                EditorEventListenerRoute::new(
                    listener.descriptor.filter.clone(),
                    Arc::clone(&listener.inbox),
                )
            })
            .collect::<Vec<_>>()
            .into();
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

    fn payload(sequence: u64) -> Arc<SharedEditorEventRecord> {
        Arc::new(SharedEditorEventRecord::new(EditorEventRecord {
            event_id: EditorEventId::new(sequence),
            sequence: EditorEventSequence::new(sequence),
            source: EditorEventSource::Headless,
            event: EditorEvent::Transient(EditorEventTransient::OpenCommandPalette),
            binding_path: None,
            operation_id: None,
            operation_display_name: None,
            operation_arguments: None,
            operation_group: None,
            transaction_id: None,
            save_generation: None,
            effects: Vec::<EditorEventEffect>::new(),
            undo_policy: EditorEventUndoPolicy::NonUndoable,
            before_revision: 0,
            after_revision: 1,
            result: EditorEventResult::success(json!(null)),
        }))
    }

    #[test]
    fn matching_listener_inboxes_share_one_immutable_payload() {
        let mut listeners = EditorEventListenerRegistry::default();
        listeners.register("a", "A").unwrap();
        listeners.register("b", "B").unwrap();
        let payload = payload(1);

        for route in listeners.delivery_routes().iter() {
            if route.accepts(payload.record()) {
                route.enqueue(Arc::clone(&payload));
            }
        }

        assert_eq!(Arc::strong_count(&payload), 3);
    }

    #[test]
    fn delivery_routes_are_rebuilt_when_listener_configuration_changes() {
        let mut listeners = EditorEventListenerRegistry::default();
        listeners.register("a", "A").unwrap();
        assert_eq!(listeners.delivery_routes().len(), 1);

        listeners.set_enabled("a", false).unwrap();
        assert!(listeners.delivery_routes().is_empty());

        listeners.set_enabled("a", true).unwrap();
        listeners
            .set_filter(
                "a",
                super::super::EditorEventListenerFilter::failures_only(),
            )
            .unwrap();
        assert_eq!(listeners.delivery_routes().len(), 1);
    }

    #[test]
    fn in_flight_route_snapshot_uses_its_captured_filter_and_new_snapshots_use_reconfiguration() {
        let mut listeners = EditorEventListenerRegistry::default();
        listeners.register("a", "A").unwrap();
        let in_flight_routes = listeners.delivery_routes();
        let event = payload(2);

        listeners
            .set_filter(
                "a",
                super::super::EditorEventListenerFilter::failures_only(),
            )
            .unwrap();
        let filtered_routes = listeners.delivery_routes();
        assert!(in_flight_routes[0].accepts(event.record()));
        assert!(!filtered_routes[0].accepts(event.record()));

        in_flight_routes[0].enqueue(Arc::clone(&event));
        for route in filtered_routes.iter() {
            if route.accepts(event.record()) {
                route.enqueue(Arc::clone(&event));
            }
        }
        assert_eq!(
            listeners
                .listener_handle("a")
                .unwrap()
                .status()
                .pending_delivery_count,
            1
        );

        listeners.set_enabled("a", false).unwrap();
        assert!(listeners.delivery_routes().is_empty());
        let detached_handle = listeners.listener_handle("a").unwrap();
        listeners.unregister("a").unwrap();
        assert!(listeners.delivery_routes().is_empty());
        assert!(listeners.listener_handle("a").is_err());

        // The snapshot acquired before unregister remains a valid in-flight owner.
        in_flight_routes[0].enqueue(event);
        assert_eq!(detached_handle.status().pending_delivery_count, 2);
    }
}
