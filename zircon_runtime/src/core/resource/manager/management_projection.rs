use std::collections::HashMap;
use std::sync::Arc;

use crate::core::resource::{
    resource_management_shard_index, ResourceId, ResourceManagementGeneration,
    ResourceManagementRow, ResourceManagementShard, ResourceRecord,
};

#[derive(Debug, Default)]
pub(super) struct ResourceManagementProjection {
    generation: Arc<ResourceManagementGeneration>,
}

impl ResourceManagementProjection {
    pub(super) fn generation(&self) -> Arc<ResourceManagementGeneration> {
        self.generation.clone()
    }

    pub(super) fn apply_delta<'a>(
        &mut self,
        removed_ids: impl IntoIterator<Item = ResourceId>,
        records: impl IntoIterator<Item = &'a ResourceRecord>,
    ) {
        let mut changed_shards =
            HashMap::<usize, HashMap<ResourceId, Arc<ResourceManagementRow>>>::new();
        let mut summary = self.generation.summary().clone();

        for id in removed_ids {
            let shard = changed_shards
                .entry(resource_management_shard_index(id))
                .or_insert_with(|| {
                    self.generation.shards()[resource_management_shard_index(id)]
                        .rows()
                        .iter()
                        .map(|row| (row.id, row.clone()))
                        .collect()
                });
            if let Some(previous) = shard.remove(&id) {
                summary.remove(&previous);
            }
        }

        for record in records {
            let shard_index = resource_management_shard_index(record.id);
            let shard = changed_shards.entry(shard_index).or_insert_with(|| {
                self.generation.shards()[shard_index]
                    .rows()
                    .iter()
                    .map(|row| (row.id, row.clone()))
                    .collect()
            });
            let next = Arc::new(ResourceManagementRow::from_record(record));
            if shard
                .get(&record.id)
                .is_some_and(|previous| previous.as_ref() == next.as_ref())
            {
                continue;
            }
            if let Some(previous) = shard.insert(record.id, next.clone()) {
                summary.remove(&previous);
            }
            summary.add(&next);
        }

        let mut shards = self.generation.shards().to_vec();
        let mut changed = false;
        for (index, rows) in changed_shards {
            let next = ResourceManagementShard::from_rows(rows.into_values().collect());
            if next.rows() != self.generation.shards()[index].rows() {
                shards[index] = Arc::new(next);
                changed = true;
            }
        }
        if !changed {
            return;
        }
        self.generation = Arc::new(ResourceManagementGeneration::from_parts(
            self.generation.sequence().wrapping_add(1),
            summary,
            shards,
        ));
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::core::resource::{
        ResourceId, ResourceKind, ResourceLocator, ResourceManagementQuery, ResourceManager,
        ResourceRecord, ResourceState,
    };

    fn record(locator: &str, kind: ResourceKind, state: ResourceState) -> ResourceRecord {
        let locator = ResourceLocator::parse(locator).unwrap();
        ResourceRecord::new(ResourceId::from_locator(&locator), kind, locator).with_state(state)
    }

    #[test]
    fn stable_resource_management_poll_reuses_the_exact_generation() {
        let manager = ResourceManager::new();
        let first = record(
            "res://models/a.glb",
            ResourceKind::Model,
            ResourceState::Ready,
        );

        manager.register_record(first.clone()).unwrap();
        let published = manager.management_generation();
        manager.register_record(first).unwrap();
        let stable = manager.management_generation();

        assert!(Arc::ptr_eq(&published, &stable));
        assert_eq!(stable.summary().total_count(), 1);
    }

    #[test]
    fn resource_management_generation_pages_in_locator_order_and_filters_without_full_records() {
        let manager = ResourceManager::new();
        manager
            .register_record(record(
                "res://textures/z.png",
                ResourceKind::Texture,
                ResourceState::Ready,
            ))
            .unwrap();
        manager
            .register_record(record(
                "res://models/b.glb",
                ResourceKind::Model,
                ResourceState::Error,
            ))
            .unwrap();
        manager
            .register_record(record(
                "res://models/a.glb",
                ResourceKind::Model,
                ResourceState::Ready,
            ))
            .unwrap();

        let generation = manager.management_generation();
        let page = generation.page(
            ResourceManagementQuery {
                kind: Some(ResourceKind::Model),
                state: None,
            },
            0,
            16,
        );

        assert_eq!(page.generation, generation.sequence());
        assert_eq!(page.total_matching_count, 2);
        assert_eq!(
            page.rows
                .iter()
                .map(|row| row.primary_locator.as_ref())
                .collect::<Vec<_>>(),
            vec!["res://models/a.glb", "res://models/b.glb"]
        );
        assert_eq!(
            generation.summary().kind(ResourceKind::Model).error_count,
            1
        );
    }

    #[test]
    fn resource_management_generation_tracks_state_rename_and_remove() {
        let manager = ResourceManager::new();
        let original = ResourceLocator::parse("res://models/a.glb").unwrap();
        let id = ResourceId::from_locator(&original);
        manager
            .register_record(ResourceRecord::new(
                id,
                ResourceKind::Model,
                original.clone(),
            ))
            .unwrap();
        let first_sequence = manager.management_generation().sequence();

        manager
            .rename(
                &original,
                ResourceLocator::parse("res://models/renamed.glb").unwrap(),
            )
            .unwrap();
        let renamed = manager.management_generation();
        assert!(renamed.sequence() > first_sequence);
        assert!(renamed.row_by_locator("res://models/a.glb").is_none());
        assert_eq!(
            renamed.row_by_id(id).unwrap().primary_locator.as_ref(),
            "res://models/renamed.glb"
        );

        manager
            .remove_by_locator(&ResourceLocator::parse("res://models/renamed.glb").unwrap())
            .unwrap();
        let removed = manager.management_generation();
        assert!(removed.row_by_id(id).is_none());
        assert_eq!(removed.summary().total_count(), 0);
    }

    #[test]
    fn lazy_registration_batch_publishes_one_generation_for_many_records() {
        let manager = ResourceManager::new();
        manager
            .register_lazy_records([
                record(
                    "res://models/a.glb",
                    ResourceKind::Model,
                    ResourceState::Ready,
                ),
                record(
                    "res://models/b.glb",
                    ResourceKind::Model,
                    ResourceState::Ready,
                ),
                record(
                    "res://textures/a.png",
                    ResourceKind::Texture,
                    ResourceState::Ready,
                ),
            ])
            .unwrap();

        let generation = manager.management_generation();
        assert_eq!(generation.sequence(), 1);
        assert_eq!(generation.summary().total_count(), 3);
    }
}
