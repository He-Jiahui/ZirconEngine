use std::collections::HashSet;

use crate::scene::ecs::{ComponentId, StorageType};

const HASH_DEDUP_COMPONENT_THRESHOLD: usize = 128;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ArchetypeSignature {
    table_components: Vec<ComponentId>,
    sparse_set_components: Vec<ComponentId>,
}

impl ArchetypeSignature {
    pub fn new(
        table_components: impl Into<Vec<ComponentId>>,
        sparse_set_components: impl Into<Vec<ComponentId>>,
    ) -> Self {
        Self {
            table_components: normalize_components(table_components.into()),
            sparse_set_components: normalize_components(sparse_set_components.into()),
        }
    }

    pub fn empty() -> Self {
        Self::new(Vec::new(), Vec::new())
    }

    pub fn contains(&self, component_id: ComponentId) -> bool {
        self.table_components.binary_search(&component_id).is_ok()
            || self
                .sparse_set_components
                .binary_search(&component_id)
                .is_ok()
    }

    pub(crate) fn with_component_added(
        &self,
        component_id: ComponentId,
        storage_type: StorageType,
    ) -> Self {
        let mut signature = self.clone();
        let components = signature.components_for_storage_mut(storage_type);
        insert_component(components, component_id);
        signature
    }

    pub(crate) fn with_component_removed(
        &self,
        component_id: ComponentId,
        storage_type: StorageType,
    ) -> Self {
        let mut signature = self.clone();
        let components = signature.components_for_storage_mut(storage_type);
        remove_component(components, component_id);
        signature
    }

    pub fn table_components(&self) -> &[ComponentId] {
        &self.table_components
    }

    pub fn sparse_set_components(&self) -> &[ComponentId] {
        &self.sparse_set_components
    }

    pub(crate) fn estimated_heap_bytes(&self) -> usize {
        self.table_components
            .capacity()
            .saturating_mul(std::mem::size_of::<ComponentId>())
            .saturating_add(
                self.sparse_set_components
                    .capacity()
                    .saturating_mul(std::mem::size_of::<ComponentId>()),
            )
    }

    pub(crate) fn ordered_component_ids(&self) -> Vec<ComponentId> {
        let mut component_ids = Vec::with_capacity(
            self.table_components
                .len()
                .saturating_add(self.sparse_set_components.len()),
        );
        let mut table_index = 0_usize;
        let mut sparse_index = 0_usize;
        while table_index < self.table_components.len()
            || sparse_index < self.sparse_set_components.len()
        {
            let next_table = self.table_components.get(table_index).copied();
            let next_sparse = self.sparse_set_components.get(sparse_index).copied();
            match (next_table, next_sparse) {
                (Some(table), Some(sparse)) if table <= sparse => {
                    component_ids.push(table);
                    table_index += 1;
                }
                (Some(_), Some(sparse)) => {
                    component_ids.push(sparse);
                    sparse_index += 1;
                }
                (Some(table), None) => {
                    component_ids.push(table);
                    table_index += 1;
                }
                (None, Some(sparse)) => {
                    component_ids.push(sparse);
                    sparse_index += 1;
                }
                (None, None) => break,
            }
        }
        component_ids
    }

    fn components_for_storage_mut(&mut self, storage_type: StorageType) -> &mut Vec<ComponentId> {
        match storage_type {
            StorageType::Table => &mut self.table_components,
            StorageType::SparseSet => &mut self.sparse_set_components,
        }
    }
}

fn normalize_components(mut components: Vec<ComponentId>) -> Vec<ComponentId> {
    if components.len() < HASH_DEDUP_COMPONENT_THRESHOLD {
        components.sort_unstable();
        components.dedup();
        return components;
    }

    let mut unique = HashSet::with_capacity(components.len());
    for component in components {
        unique.insert(component);
    }
    let mut components = unique.into_iter().collect::<Vec<_>>();
    components.sort_unstable();
    components
}

fn insert_component(components: &mut Vec<ComponentId>, component_id: ComponentId) {
    if let Err(index) = components.binary_search(&component_id) {
        components.insert(index, component_id);
    }
}

fn remove_component(components: &mut Vec<ComponentId>, component_id: ComponentId) {
    if let Ok(index) = components.binary_search(&component_id) {
        components.remove(index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_membership_updates_only_the_known_storage_partition() {
        let signature =
            ArchetypeSignature::new(vec![ComponentId::new(4)], vec![ComponentId::new(2)]);

        let updated = signature
            .with_component_added(ComponentId::new(3), StorageType::Table)
            .with_component_removed(ComponentId::new(2), StorageType::SparseSet);

        assert_eq!(
            updated.table_components(),
            &[ComponentId::new(3), ComponentId::new(4)]
        );
        assert!(updated.sparse_set_components().is_empty());
        assert_eq!(signature.table_components(), &[ComponentId::new(4)]);
        assert_eq!(signature.sparse_set_components(), &[ComponentId::new(2)]);
    }

    #[test]
    fn component_membership_updates_are_idempotent() {
        let signature = ArchetypeSignature::empty()
            .with_component_added(ComponentId::new(7), StorageType::SparseSet)
            .with_component_added(ComponentId::new(7), StorageType::SparseSet)
            .with_component_removed(ComponentId::new(8), StorageType::SparseSet);

        assert_eq!(signature.sparse_set_components(), &[ComponentId::new(7)]);
    }

    #[test]
    fn ordered_component_ids_merge_table_and_sparse_partitions() {
        let signature = ArchetypeSignature::new(
            vec![ComponentId::new(2), ComponentId::new(6)],
            vec![ComponentId::new(1), ComponentId::new(4)],
        );

        assert_eq!(
            signature.ordered_component_ids(),
            vec![
                ComponentId::new(1),
                ComponentId::new(2),
                ComponentId::new(4),
                ComponentId::new(6),
            ]
        );
    }
}

#[cfg(test)]
#[path = "signature/hash_dedup_tests.rs"]
mod hash_dedup_tests;
