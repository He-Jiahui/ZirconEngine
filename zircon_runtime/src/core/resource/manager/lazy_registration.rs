use crate::core::resource::{
    ResourceEvent, ResourceEventKind, ResourceRecord, ResourceState, RuntimeResourceState,
    UntypedResourceHandle,
};

use super::resource_manager::ResourceManager;
use super::revision::next_ready_revision;

impl ResourceManager {
    pub fn register_lazy_record(&self, record: ResourceRecord) -> UntypedResourceHandle {
        self.register_lazy_records(std::iter::once(record))
            .pop()
            .expect("one lazy record produces one handle")
    }

    pub fn register_lazy_records(
        &self,
        records: impl IntoIterator<Item = ResourceRecord>,
    ) -> Vec<UntypedResourceHandle> {
        let mut outcomes = {
            let mut registry = self.lock_registry_write();
            let mut outcomes = Vec::new();
            for mut record in records {
                let id = record.id;
                let (event_kind, invalidate_payload) = match registry.get(id) {
                    Some(previous) => {
                        let previous_state = previous.state;
                        let previous_revision = previous.revision;
                        if record.state == ResourceState::Ready {
                            record.revision = next_ready_revision(previous, &record);
                        } else {
                            record.revision = previous_revision;
                        }
                        let metadata_changed = previous != &record;
                        let content_changed = record.revision != previous_revision
                            || previous_state != ResourceState::Ready
                            || record.state != ResourceState::Ready;
                        (
                            metadata_changed.then_some(ResourceEventKind::Updated),
                            content_changed,
                        )
                    }
                    None => {
                        if record.state == ResourceState::Ready && record.revision == 0 {
                            record.revision = 1;
                        }
                        (Some(ResourceEventKind::Added), true)
                    }
                };
                registry.upsert_registry_only(record.clone());
                outcomes.push((record, event_kind, invalidate_payload));
            }
            registry.publish_records(outcomes.iter().map(|(record, _, _)| record));
            outcomes
        };

        for (record, _, invalidate_payload) in &outcomes {
            self.ensure_runtime_slot(record.id);
            if *invalidate_payload {
                self.lock_payloads_write().remove(&record.id);
                let runtime_state = if record.state == ResourceState::Error {
                    RuntimeResourceState::Error
                } else {
                    RuntimeResourceState::Unloaded
                };
                self.set_runtime_state(record.id, runtime_state);
            }
        }

        self.refresh_readiness_many(outcomes.iter().map(|(record, _, _)| record.id));

        for (record, event_kind, _) in &outcomes {
            if let Some(event_kind) = event_kind {
                self.broadcast(ResourceEvent {
                    kind: *event_kind,
                    resource_kind: record.kind,
                    id: record.id,
                    locator: Some(record.primary_locator.clone()),
                    previous_locator: None,
                    revision: record.revision,
                });
            }
        }

        outcomes
            .drain(..)
            .map(|(record, _, _)| UntypedResourceHandle::new(record.id, record.kind))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::core::resource::{
        ResourceId, ResourceKind, ResourceLocator, ResourceRecord, ResourceState,
        RuntimeResourceState,
    };

    use super::ResourceManager;

    #[derive(Debug)]
    struct TestPayload;

    #[test]
    fn lazy_registration_preserves_unchanged_payload_and_invalidates_changed_content() {
        let manager = ResourceManager::new();
        let record = ready_record("digest-v1");
        let id = record.id;

        manager.register_lazy_record(record.clone());
        assert_eq!(manager.registry().get(id).unwrap().revision, 1);
        assert_eq!(
            manager.runtime_state(id),
            Some(RuntimeResourceState::Unloaded)
        );
        assert!(manager.store_payload(id, TestPayload));

        manager.register_lazy_record(record.clone());
        assert!(manager.get_untyped(id).is_some());
        assert_eq!(manager.registry().get(id).unwrap().revision, 1);

        let mut changed = record;
        changed.source_hash = "digest-v2".to_string();
        manager.register_lazy_record(changed);
        assert!(manager.get_untyped(id).is_none());
        assert_eq!(manager.registry().get(id).unwrap().revision, 2);
        assert_eq!(
            manager.runtime_state(id),
            Some(RuntimeResourceState::Unloaded)
        );
    }

    #[test]
    fn lazy_registration_evicts_payload_when_metadata_becomes_error() {
        let manager = ResourceManager::new();
        let ready = ready_record("digest-v1");
        let id = ready.id;
        manager.register_lazy_record(ready.clone());
        assert!(manager.store_payload(id, TestPayload));

        let mut failed = ready;
        failed.state = ResourceState::Error;
        manager.register_lazy_record(failed);

        assert!(manager.get_untyped(id).is_none());
        assert_eq!(manager.runtime_state(id), Some(RuntimeResourceState::Error));
    }

    fn ready_record(source_hash: &str) -> ResourceRecord {
        let locator = ResourceLocator::parse("res://data/lazy.json").unwrap();
        ResourceRecord::new(
            ResourceId::from_locator(&locator),
            ResourceKind::Data,
            locator,
        )
        .with_source_hash(source_hash)
        .with_state(ResourceState::Ready)
    }
}
