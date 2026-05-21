use std::collections::HashMap;

use crate::scene::ecs::{ArchetypeId, ArchetypeSignature, ComponentId};
use crate::scene::EntityId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchetypeRecord {
    id: ArchetypeId,
    signature: ArchetypeSignature,
    entities: Vec<EntityId>,
}

impl ArchetypeRecord {
    pub fn id(&self) -> ArchetypeId {
        self.id
    }

    pub fn signature(&self) -> &ArchetypeSignature {
        &self.signature
    }

    pub fn entities(&self) -> &[EntityId] {
        &self.entities
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArchetypeMove {
    pub entity_row: usize,
    pub swapped_entity: Option<(EntityId, usize)>,
}

#[derive(Clone, Debug)]
pub struct ArchetypeIndex {
    records: Vec<ArchetypeRecord>,
    by_signature: HashMap<ArchetypeSignature, ArchetypeId>,
    by_component: HashMap<ComponentId, Vec<ArchetypeId>>,
}

impl ArchetypeIndex {
    pub fn new() -> Self {
        let empty = ArchetypeSignature::empty();
        let mut by_signature = HashMap::new();
        by_signature.insert(empty.clone(), ArchetypeId::EMPTY);
        Self {
            records: vec![ArchetypeRecord {
                id: ArchetypeId::EMPTY,
                signature: empty,
                entities: Vec::new(),
            }],
            by_signature,
            by_component: HashMap::new(),
        }
    }

    pub fn generation(&self) -> u64 {
        self.records.len() as u64
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn signature(&self, id: ArchetypeId) -> Option<&ArchetypeSignature> {
        self.records.get(id.index()).map(ArchetypeRecord::signature)
    }

    pub fn entities(&self, id: ArchetypeId) -> Option<&[EntityId]> {
        self.records.get(id.index()).map(ArchetypeRecord::entities)
    }

    pub fn id_or_insert(&mut self, signature: ArchetypeSignature) -> ArchetypeId {
        if let Some(id) = self.by_signature.get(&signature).copied() {
            return id;
        }

        let id = ArchetypeId::new(self.records.len());
        self.index_signature_components(id, &signature);
        self.records.push(ArchetypeRecord {
            id,
            signature: signature.clone(),
            entities: Vec::new(),
        });
        self.by_signature.insert(signature, id);
        id
    }

    pub fn move_entity(
        &mut self,
        entity: EntityId,
        previous: Option<ArchetypeId>,
        target: ArchetypeId,
    ) -> ArchetypeMove {
        if previous == Some(target) {
            return ArchetypeMove {
                entity_row: self.add_entity_to(target, entity),
                swapped_entity: None,
            };
        }

        let swapped_entity = previous.and_then(|id| self.remove_entity_from(id, entity));
        let entity_row = self.add_entity_to(target, entity);
        ArchetypeMove {
            entity_row,
            swapped_entity,
        }
    }

    pub fn matching_archetypes(
        &self,
        required: &[ComponentId],
        without: &[ComponentId],
    ) -> Vec<ArchetypeId> {
        let mut candidates = if let Some(component_id) = required
            .iter()
            .min_by_key(|component_id| self.by_component.get(*component_id).map_or(0, Vec::len))
        {
            self.by_component
                .get(component_id)
                .cloned()
                .unwrap_or_default()
        } else {
            self.records.iter().map(ArchetypeRecord::id).collect()
        };

        candidates.retain(|id| {
            self.signature(*id).is_some_and(|signature| {
                required
                    .iter()
                    .all(|component_id| signature.contains(*component_id))
                    && without
                        .iter()
                        .all(|component_id| !signature.contains(*component_id))
            })
        });
        candidates.sort_unstable();
        candidates.dedup();
        candidates
    }

    fn add_entity_to(&mut self, id: ArchetypeId, entity: EntityId) -> usize {
        let Some(record) = self.records.get_mut(id.index()) else {
            return 0;
        };
        if let Some(row) = record
            .entities
            .iter()
            .position(|current| *current == entity)
        {
            return row;
        }
        let row = record.entities.len();
        record.entities.push(entity);
        row
    }

    fn remove_entity_from(
        &mut self,
        id: ArchetypeId,
        entity: EntityId,
    ) -> Option<(EntityId, usize)> {
        let record = self.records.get_mut(id.index())?;
        let row = record
            .entities
            .iter()
            .position(|current| *current == entity)?;
        let last_row = record.entities.len() - 1;
        let removed = record.entities.swap_remove(row);
        debug_assert_eq!(removed, entity);
        (row != last_row).then(|| (record.entities[row], row))
    }

    fn index_signature_components(&mut self, id: ArchetypeId, signature: &ArchetypeSignature) {
        for component_id in signature
            .table_components()
            .iter()
            .chain(signature.sparse_set_components())
            .copied()
        {
            let ids = self.by_component.entry(component_id).or_default();
            if !ids.contains(&id) {
                ids.push(id);
                ids.sort_unstable();
            }
        }
    }
}

impl Default for ArchetypeIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for ArchetypeIndex {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for ArchetypeIndex {}
