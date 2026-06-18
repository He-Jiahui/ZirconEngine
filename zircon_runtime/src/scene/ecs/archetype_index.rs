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
        let record = self.records.get(id.index())?;
        Some(record.signature())
    }

    pub fn entities(&self, id: ArchetypeId) -> Option<&[EntityId]> {
        let record = self.records.get(id.index())?;
        Some(record.entities())
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

        let swapped_entity = if let Some(id) = previous {
            self.remove_entity_from(id, entity)
        } else {
            None
        };
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
        let mut candidates = match self.shortest_required_archetype_ids(required) {
            Some(ids) => ids.to_vec(),
            None => Self::all_archetype_ids(&self.records),
        };
        candidates.retain(|id| self.archetype_matches_required_without(*id, required, without));
        candidates
    }

    fn shortest_required_archetype_ids(&self, required: &[ComponentId]) -> Option<&[ArchetypeId]> {
        let mut selected = None;
        let mut selected_len = usize::MAX;
        for component_id in required {
            let ids = match self.by_component.get(component_id) {
                Some(ids) => ids.as_slice(),
                None => return Some(&[]),
            };
            let candidate_len = ids.len();
            if selected.is_none() || candidate_len < selected_len {
                selected = Some(ids);
                selected_len = candidate_len;
                if candidate_len == 0 {
                    break;
                }
            }
        }
        selected
    }

    fn archetype_matches_required_without(
        &self,
        id: ArchetypeId,
        required: &[ComponentId],
        without: &[ComponentId],
    ) -> bool {
        let Some(record) = self.records.get(id.index()) else {
            return false;
        };
        let signature = record.signature();
        for component_id in required {
            if !signature.contains(*component_id) {
                return false;
            }
        }
        for component_id in without {
            if signature.contains(*component_id) {
                return false;
            }
        }
        true
    }

    fn all_archetype_ids(records: &[ArchetypeRecord]) -> Vec<ArchetypeId> {
        let mut ids = Vec::with_capacity(records.len());
        for record in records {
            ids.push(record.id());
        }
        ids
    }

    fn add_entity_to(&mut self, id: ArchetypeId, entity: EntityId) -> usize {
        let Some(record) = self.records.get_mut(id.index()) else {
            return 0;
        };
        if let Some(row) = entity_row(&record.entities, entity) {
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
        let row = entity_row(&record.entities, entity)?;
        let last_row = record.entities.len() - 1;
        let removed = record.entities.swap_remove(row);
        debug_assert_eq!(removed, entity);
        if row != last_row {
            Some((record.entities[row], row))
        } else {
            None
        }
    }

    fn index_signature_components(&mut self, id: ArchetypeId, signature: &ArchetypeSignature) {
        for component_id in signature
            .table_components()
            .iter()
            .chain(signature.sparse_set_components())
            .copied()
        {
            let ids = self.by_component.entry(component_id).or_default();
            insert_archetype_id(ids, id);
        }
    }
}

fn insert_archetype_id(ids: &mut Vec<ArchetypeId>, id: ArchetypeId) {
    if let Err(index) = ids.binary_search(&id) {
        ids.insert(index, id);
    }
}

fn entity_row(entities: &[EntityId], entity: EntityId) -> Option<usize> {
    let mut row = 0;
    while row < entities.len() {
        if entities[row] == entity {
            return Some(row);
        }
        row += 1;
    }
    None
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
