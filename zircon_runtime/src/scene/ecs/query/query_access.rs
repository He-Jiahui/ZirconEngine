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
        !self.has_disjoint_filter(other)
            && (sorted_component_slices_intersect(&self.writes, &other.reads)
                || sorted_component_slices_intersect(&self.reads, &other.writes)
                || sorted_component_slices_intersect(&self.writes, &other.writes))
    }

    pub fn conflicting_components_with(&self, other: &Self) -> Vec<ComponentId> {
        if self.has_disjoint_filter(other) {
            return Vec::new();
        }

        let mut conflicts = Vec::new();
        push_sorted_component_intersections(&mut conflicts, &self.writes, &other.reads);
        push_sorted_component_intersections(&mut conflicts, &self.reads, &other.writes);
        push_sorted_component_intersections(&mut conflicts, &self.writes, &other.writes);
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

fn push_sorted_component_intersections(
    conflicts: &mut Vec<ComponentId>,
    left: &[ComponentId],
    right: &[ComponentId],
) {
    let mut left_index = 0;
    let mut right_index = 0;
    while let (Some(left_value), Some(right_value)) = (left.get(left_index), right.get(right_index))
    {
        if left_value == right_value {
            insert_sorted_component_id(conflicts, *left_value);
            left_index += 1;
            right_index += 1;
        } else if left_value < right_value {
            left_index += 1;
        } else {
            right_index += 1;
        }
    }
}

fn insert_id(ids: &mut Vec<ComponentId>, component_id: ComponentId) {
    insert_sorted_component_id(ids, component_id);
}

fn insert_sorted_component_id(ids: &mut Vec<ComponentId>, component_id: ComponentId) {
    if let Err(index) = ids.binary_search(&component_id) {
        ids.insert(index, component_id);
    }
}

fn contains_id(ids: &[ComponentId], component_id: ComponentId) -> bool {
    ids.binary_search(&component_id).is_ok()
}

fn intersects(left: &[ComponentId], right: &[ComponentId]) -> bool {
    sorted_component_slices_intersect(left, right)
}

fn sorted_component_slices_intersect(left: &[ComponentId], right: &[ComponentId]) -> bool {
    let mut left_index = 0;
    let mut right_index = 0;
    while let (Some(left_value), Some(right_value)) = (left.get(left_index), right.get(right_index))
    {
        if left_value == right_value {
            return true;
        }
        if left_value < right_value {
            left_index += 1;
        } else {
            right_index += 1;
        }
    }
    false
}
