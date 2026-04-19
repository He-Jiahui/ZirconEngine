use super::super::VirtualGeometryRuntimeState;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum PageRelation {
    Unrelated,
    Ancestor,
    Descendant,
}

impl VirtualGeometryRuntimeState {
    pub(in crate::runtime::virtual_geometry) fn ordered_evictable_pages_for_target(
        &self,
        target_page_id: u32,
        evictable_pages: &[u32],
    ) -> Vec<u32> {
        let mut ordered = evictable_pages.to_vec();
        ordered.sort_by_key(|page_id| {
            let (relation, lineage_distance) =
                self.page_relation_to_target(target_page_id, *page_id);
            let active_request_lineage_priority =
                self.active_request_lineage_priority(target_page_id, *page_id);
            let relation_order = match relation {
                PageRelation::Unrelated => 0_u8,
                PageRelation::Ancestor => 1_u8,
                PageRelation::Descendant => 2_u8,
            };
            let active_request_group = u8::from(active_request_lineage_priority.is_some());
            let active_request_order = active_request_lineage_priority
                .map(|(request_order, _lineage_distance)| usize::MAX - request_order)
                .unwrap_or_default();
            let active_request_distance = active_request_lineage_priority
                .map(|(_request_order, lineage_distance)| u32::MAX - lineage_distance)
                .unwrap_or_default();
            let hot_frontier_group = u8::from(self.page_or_lineage_is_hot(*page_id));
            let relation_distance_order =
                descendant_frontier_distance_order(relation, lineage_distance, hot_frontier_group);
            (
                relation_order,
                active_request_group,
                hot_frontier_group,
                active_request_order,
                relation_distance_order,
                active_request_distance,
                *page_id,
            )
        });
        ordered
    }

    fn page_relation_to_target(
        &self,
        target_page_id: u32,
        candidate_page_id: u32,
    ) -> (PageRelation, u32) {
        if let Some(distance) = self.page_lineage_distance(candidate_page_id, target_page_id) {
            return (PageRelation::Ancestor, distance);
        }
        if let Some(distance) = self.page_lineage_distance(target_page_id, candidate_page_id) {
            return (PageRelation::Descendant, distance);
        }
        (PageRelation::Unrelated, 0)
    }

    fn active_request_lineage_priority(
        &self,
        target_page_id: u32,
        candidate_page_id: u32,
    ) -> Option<(usize, u32)> {
        self.current_requested_page_order
            .iter()
            .filter(|(requested_page_id, _)| self.pending_pages.contains(requested_page_id))
            .filter(|(requested_page_id, _)| **requested_page_id != target_page_id)
            .filter_map(|(requested_page_id, request_order)| {
                let lineage_distance = if *requested_page_id == candidate_page_id {
                    Some(0)
                } else {
                    self.page_lineage_distance(candidate_page_id, *requested_page_id)
                        .or_else(|| {
                            self.page_lineage_distance(*requested_page_id, candidate_page_id)
                        })
                }?;
                Some((*request_order, lineage_distance))
            })
            .min_by_key(|(request_order, lineage_distance)| (*request_order, *lineage_distance))
    }

    fn page_lineage_distance(&self, ancestor_page_id: u32, descendant_page_id: u32) -> Option<u32> {
        if ancestor_page_id == descendant_page_id {
            return None;
        }

        let mut current_page_id = descendant_page_id;
        let mut distance = 0_u32;
        while let Some(parent_page_id) = self.page_parent_pages.get(&current_page_id).copied() {
            distance = distance.saturating_add(1);
            if parent_page_id == ancestor_page_id {
                return Some(distance);
            }
            current_page_id = parent_page_id;
        }

        None
    }

    pub(in crate::runtime::virtual_geometry) fn page_or_lineage_is_hot(
        &self,
        page_id: u32,
    ) -> bool {
        if self.page_is_frontier_hot(page_id) {
            return true;
        }

        let mut current_page_id = page_id;
        while let Some(parent_page_id) = self.page_parent_pages.get(&current_page_id).copied() {
            if self.page_is_frontier_hot(parent_page_id) {
                return true;
            }
            current_page_id = parent_page_id;
        }

        let mut stack = self
            .page_parent_pages
            .iter()
            .filter_map(|(&candidate_page_id, &parent_page_id)| {
                (parent_page_id == page_id).then_some(candidate_page_id)
            })
            .collect::<Vec<_>>();
        let mut visited_page_ids = BTreeSet::new();

        while let Some(candidate_page_id) = stack.pop() {
            if !visited_page_ids.insert(candidate_page_id) {
                continue;
            }
            if self.page_is_frontier_hot(candidate_page_id) {
                return true;
            }
            stack.extend(self.page_parent_pages.iter().filter_map(
                |(&descendant_page_id, &parent_page_id)| {
                    (parent_page_id == candidate_page_id).then_some(descendant_page_id)
                },
            ));
        }

        false
    }

    pub(in crate::runtime::virtual_geometry) fn page_is_frontier_hot(&self, page_id: u32) -> bool {
        self.current_hot_resident_pages.contains(&page_id)
            || self.recent_hot_resident_pages.contains(&page_id)
    }

    pub(in crate::runtime::virtual_geometry) fn frontier_hot_resident_pages(
        &self,
    ) -> BTreeSet<u32> {
        self.current_hot_resident_pages
            .iter()
            .chain(self.recent_hot_resident_pages.iter())
            .copied()
            .collect()
    }
}

fn descendant_frontier_distance_order(
    relation: PageRelation,
    lineage_distance: u32,
    hot_frontier_group: u8,
) -> u32 {
    if matches!(relation, PageRelation::Descendant) && hot_frontier_group != 0 {
        return lineage_distance;
    }

    u32::MAX - lineage_distance
}
