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
        // Every write is mirrored in reads, so an absent read is also absent from writes.
        let read_index = match self.reads.binary_search(&component_id) {
            Ok(_) => return Err(QueryAccessError::ConflictingComponentAccess { component_id }),
            Err(index) => index,
        };
        self.reads.insert(read_index, component_id);
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
                || sorted_component_slices_intersect(&self.reads, &other.writes))
    }

    pub fn conflicting_components_with(&self, other: &Self) -> Vec<ComponentId> {
        if self.has_disjoint_filter(other) {
            return Vec::new();
        }

        let mut conflicts = Vec::with_capacity(component_conflict_upper_bound(self, other));
        push_sorted_component_intersections(&mut conflicts, &self.writes, &other.reads);
        // Writes are mirrored into reads. The first pass already covers write/write
        // conflicts, so this pass scans only read-only IDs to avoid duplicate work.
        push_read_only_component_intersections(
            &mut conflicts,
            &self.reads,
            &self.writes,
            &other.writes,
        );
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

fn component_conflict_upper_bound(left: &QueryAccess, right: &QueryAccess) -> usize {
    left.writes.len().min(right.reads.len())
        + read_only_component_count(left).min(right.writes.len())
}

fn read_only_component_count(access: &QueryAccess) -> usize {
    access.reads.len().saturating_sub(access.writes.len())
}

fn push_sorted_component_intersections(
    conflicts: &mut Vec<ComponentId>,
    left: &[ComponentId],
    right: &[ComponentId],
) {
    let mut left_index = 0;
    let mut right_index = 0;
    while left_index < left.len() && right_index < right.len() {
        let left_value = left[left_index];
        let right_value = right[right_index];
        if left_value == right_value {
            insert_sorted_component_id(conflicts, left_value);
            left_index += 1;
            right_index += 1;
        } else if left_value < right_value {
            left_index += 1;
        } else {
            right_index += 1;
        }
    }
}

fn push_read_only_component_intersections(
    conflicts: &mut Vec<ComponentId>,
    reads: &[ComponentId],
    writes: &[ComponentId],
    right: &[ComponentId],
) {
    let mut read_index = 0;
    let mut write_index = 0;
    let mut right_index = 0;
    while read_index < reads.len() && right_index < right.len() {
        let read_value = reads[read_index];
        let right_value = right[right_index];
        if read_component_is_written(read_value, writes, &mut write_index) {
            read_index += 1;
            continue;
        }
        if read_value == right_value {
            insert_sorted_component_id(conflicts, read_value);
            read_index += 1;
            right_index += 1;
        } else if read_value < right_value {
            read_index += 1;
        } else {
            right_index += 1;
        }
    }
}

fn read_component_is_written(
    component_id: ComponentId,
    writes: &[ComponentId],
    write_index: &mut usize,
) -> bool {
    while *write_index < writes.len() {
        let write_value = writes[*write_index];
        if write_value < component_id {
            *write_index += 1;
            continue;
        }
        if write_value == component_id {
            *write_index += 1;
            return true;
        }
        return false;
    }
    false
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
    while left_index < left.len() && right_index < right.len() {
        let left_value = left[left_index];
        let right_value = right[right_index];
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

#[cfg(test)]
mod tests {
    use super::{QueryAccess, QueryAccessError};
    use crate::scene::ecs::ComponentId;

    fn assert_writes_are_mirrored(access: &QueryAccess) {
        assert!(access
            .writes()
            .iter()
            .all(|component_id| access.reads().binary_search(component_id).is_ok()));
    }

    #[test]
    fn runtime60_batch_writes_remain_mirrored_in_reads() {
        let mut access = QueryAccess::default();
        for index in [7, 1, 4] {
            access
                .add_write(ComponentId::new(index))
                .expect("distinct writes should be accepted");
        }

        assert_eq!(
            access.reads(),
            [
                ComponentId::new(1),
                ComponentId::new(4),
                ComponentId::new(7)
            ]
        );
        assert_eq!(access.writes(), access.reads());
        assert_writes_are_mirrored(&access);
    }

    #[test]
    fn runtime60_batch_merged_writes_remain_mirrored_in_reads() {
        let mut access = QueryAccess::default();
        access
            .add_write(ComponentId::new(9))
            .expect("initial write should be accepted");

        let mut other = QueryAccess::default();
        other.add_filter_read(ComponentId::new(8));
        other
            .add_write(ComponentId::new(4))
            .expect("distinct write should be accepted");
        access.merge_param_set_unchecked(&other);

        assert_writes_are_mirrored(&access);
        assert_eq!(
            access.reads(),
            [
                ComponentId::new(4),
                ComponentId::new(8),
                ComponentId::new(9)
            ]
        );
        assert_eq!(access.writes(), [ComponentId::new(4), ComponentId::new(9)]);
    }

    #[test]
    fn runtime60_batch_repeated_write_keeps_the_conflict_diagnostic() {
        let component_id = ComponentId::new(12);
        let mut access = QueryAccess::default();
        access
            .add_write(component_id)
            .expect("first write should be accepted");

        assert_eq!(
            access.add_write(component_id),
            Err(QueryAccessError::ConflictingComponentAccess { component_id })
        );
        assert_eq!(access.reads(), [component_id]);
        assert_eq!(access.writes(), [component_id]);
    }
}
