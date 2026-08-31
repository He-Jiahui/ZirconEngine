use crate::{ResourceMutationBatch, ResourceRecord, ResourceResult, UntypedResourceHandle};

use super::resource_manager::ResourceManager;

impl ResourceManager {
    pub fn register_lazy_record(
        &self,
        record: ResourceRecord,
    ) -> ResourceResult<UntypedResourceHandle> {
        Ok(self
            .register_lazy_records(std::iter::once(record))?
            .pop()
            .expect("one lazy record produces one handle"))
    }

    pub fn register_lazy_records(
        &self,
        records: impl IntoIterator<Item = ResourceRecord>,
    ) -> ResourceResult<Vec<UntypedResourceHandle>> {
        let records = records.into_iter();
        let (lower_bound, _) = records.size_hint();
        let mut ids = Vec::with_capacity(lower_bound);
        let mut batch = ResourceMutationBatch::new();
        for record in records {
            ids.push(record.id);
            batch = batch.upsert_lazy(record);
        }
        let receipt = self.commit(batch)?;
        Ok(ids
            .into_iter()
            .map(|id| {
                receipt
                    .handle(id)
                    .expect("a committed lazy upsert produces a handle")
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
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

        manager
            .register_lazy_record(record.clone())
            .expect("register initial record");
        assert_eq!(manager.registry().get(id).unwrap().revision, 1);
        assert_eq!(
            manager.runtime_state(id),
            Some(RuntimeResourceState::Unloaded)
        );
        manager
            .store_payload(id, 1, TestPayload)
            .expect("store initial payload");

        manager
            .register_lazy_record(record.clone())
            .expect("register unchanged record");
        assert!(manager.get_untyped(id).is_some());
        assert_eq!(manager.registry().get(id).unwrap().revision, 1);

        let mut changed = record;
        changed.source_hash = "digest-v2".to_string();
        manager
            .register_lazy_record(changed)
            .expect("register changed record");
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
        manager
            .register_lazy_record(ready.clone())
            .expect("register ready record");
        manager
            .store_payload(id, 1, TestPayload)
            .expect("store ready payload");

        let mut failed = ready;
        failed.state = ResourceState::Error;
        manager
            .register_lazy_record(failed)
            .expect("register failed record");

        assert!(manager.get_untyped(id).is_none());
        assert_eq!(manager.runtime_state(id), Some(RuntimeResourceState::Error));
    }

    #[test]
    fn lazy_batch_registration_preserves_input_handle_order() {
        let manager = ResourceManager::new();
        let first = ready_record("digest-first");
        let second_locator = ResourceLocator::parse("res://data/second.json").unwrap();
        let second = ResourceRecord::new(
            ResourceId::from_locator(&second_locator),
            ResourceKind::Data,
            second_locator,
        )
        .with_source_hash("digest-second")
        .with_state(ResourceState::Ready);

        let handles = manager
            .register_lazy_records([first.clone(), second.clone()])
            .expect("register lazy batch");

        assert_eq!(
            handles
                .into_iter()
                .map(|handle| handle.id())
                .collect::<Vec<_>>(),
            vec![first.id, second.id]
        );
    }

    #[test]
    fn lazy_batch_registration_does_not_buffer_all_records_before_building_the_batch() {
        let source = include_str!("lazy_registration.rs");
        let registration = source
            .split("pub fn register_lazy_records")
            .nth(1)
            .and_then(|source| source.split("let receipt = self.commit(batch)?;").next())
            .expect("lazy batch registration implementation");

        assert!(registration.contains("let records = records.into_iter();"));
        assert!(!registration.contains("records.into_iter().collect::<Vec<_>>()"));
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
