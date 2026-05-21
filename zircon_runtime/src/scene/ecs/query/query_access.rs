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
        Ok(())
    }

    pub fn add_filter_read(&mut self, component_id: ComponentId) {
        insert_id(&mut self.reads, component_id);
    }

    pub fn add_write(&mut self, component_id: ComponentId) -> Result<(), QueryAccessError> {
        if contains_id(&self.reads, component_id) || contains_id(&self.writes, component_id) {
            return Err(QueryAccessError::ConflictingComponentAccess { component_id });
        }
        insert_id(&mut self.reads, component_id);
        insert_id(&mut self.writes, component_id);
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
        !self.conflicting_components_with(other).is_empty()
    }

    pub fn conflicting_components_with(&self, other: &Self) -> Vec<ComponentId> {
        if self.has_disjoint_filter(other) {
            return Vec::new();
        }

        let mut conflicts = Vec::new();
        push_intersections(&mut conflicts, &self.writes, &other.reads);
        push_intersections(&mut conflicts, &self.reads, &other.writes);
        push_intersections(&mut conflicts, &self.writes, &other.writes);
        conflicts
    }

    pub(crate) fn merge_param_set_unchecked(&mut self, other: &Self) {
        for component_id in other.reads.iter().copied() {
            insert_id(&mut self.reads, component_id);
        }
        for component_id in other.writes.iter().copied() {
            insert_id(&mut self.writes, component_id);
        }
    }

    fn has_disjoint_filter(&self, other: &Self) -> bool {
        intersects(&self.with, &other.without) || intersects(&self.without, &other.with)
    }
}

fn push_intersections(
    conflicts: &mut Vec<ComponentId>,
    left: &[ComponentId],
    right: &[ComponentId],
) {
    for component_id in left {
        if contains_id(right, *component_id) {
            insert_id(conflicts, *component_id);
        }
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
