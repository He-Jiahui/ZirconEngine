use crate::scene::ecs::ComponentId;

use super::QueryAccessError;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct QueryAccess {
    reads: Vec<ComponentId>,
    writes: Vec<ComponentId>,
    with: Vec<ComponentId>,
    without: Vec<ComponentId>,
}

impl QueryAccess {
    pub fn add_read(&mut self, component_id: ComponentId) -> Result<(), QueryAccessError> {
        if contains_id(&self.writes, component_id) {
            return Err(QueryAccessError::ConflictingComponentAccess { component_id });
        }
        insert_id(&mut self.reads, component_id);
        insert_id(&mut self.with, component_id);
        Ok(())
    }

    pub fn add_write(&mut self, component_id: ComponentId) -> Result<(), QueryAccessError> {
        if contains_id(&self.reads, component_id) || contains_id(&self.writes, component_id) {
            return Err(QueryAccessError::ConflictingComponentAccess { component_id });
        }
        insert_id(&mut self.reads, component_id);
        insert_id(&mut self.writes, component_id);
        insert_id(&mut self.with, component_id);
        Ok(())
    }

    pub fn add_with(&mut self, component_id: ComponentId) {
        insert_id(&mut self.with, component_id);
    }

    pub fn add_without(&mut self, component_id: ComponentId) {
        insert_id(&mut self.without, component_id);
    }

    pub fn reads(&self) -> &[ComponentId] {
        &self.reads
    }

    pub fn writes(&self) -> &[ComponentId] {
        &self.writes
    }

    pub fn with(&self) -> &[ComponentId] {
        &self.with
    }

    pub fn without(&self) -> &[ComponentId] {
        &self.without
    }

    pub fn conflicts_with(&self, other: &Self) -> bool {
        if !self.has_data_conflict(other) {
            return false;
        }
        !self.has_disjoint_filter(other)
    }

    fn has_data_conflict(&self, other: &Self) -> bool {
        intersects(&self.writes, &other.reads)
            || intersects(&self.reads, &other.writes)
            || intersects(&self.writes, &other.writes)
    }

    fn has_disjoint_filter(&self, other: &Self) -> bool {
        intersects(&self.with, &other.without) || intersects(&self.without, &other.with)
    }
}

fn insert_id(ids: &mut Vec<ComponentId>, component_id: ComponentId) {
    if !contains_id(ids, component_id) {
        ids.push(component_id);
        ids.sort_unstable();
    }
}

fn contains_id(ids: &[ComponentId], component_id: ComponentId) -> bool {
    ids.binary_search(&component_id).is_ok()
}

fn intersects(left: &[ComponentId], right: &[ComponentId]) -> bool {
    left.iter()
        .any(|component_id| contains_id(right, *component_id))
}
