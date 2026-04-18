use std::{cmp::Reverse, collections::BTreeMap};

use crate::types::VirtualGeometryPrepareRequest;

use super::super::{VirtualGeometryPageRequest, VirtualGeometryRuntimeState};
use super::available_slots::available_slots;

#[derive(Clone, Copy)]
struct AssignedSlotPlan {
    slot: Option<u32>,
    recycled_page_id: Option<u32>,
}

pub(super) fn pending_page_requests(
    state: &VirtualGeometryRuntimeState,
) -> Vec<VirtualGeometryPrepareRequest> {
    let mut pending_requests = state
        .pending_requests
        .iter()
        .filter(|request| !has_pending_ancestor_request(state, request.page_id))
        .cloned()
        .collect::<Vec<_>>();
    pending_requests.sort_by_key(|request| {
        (
            current_request_rank(state, request.page_id),
            Reverse(resident_descendant_count(state, request.page_id)),
            Reverse(descendant_count(state, request.page_id)),
            page_depth(state, request.page_id),
            request.generation,
            request.page_id,
        )
    });
    let assigned_slots = assigned_slots(state, &pending_requests);

    pending_requests
        .into_iter()
        .map(|request| VirtualGeometryPrepareRequest {
            page_id: request.page_id,
            size_bytes: request.size_bytes,
            generation: request.generation,
            frontier_rank: current_request_rank(state, request.page_id).min(u32::MAX as usize)
                as u32,
            assigned_slot: assigned_slots
                .get(&request.page_id)
                .and_then(|plan| plan.slot),
            recycled_page_id: assigned_slots
                .get(&request.page_id)
                .and_then(|plan| plan.recycled_page_id),
        })
        .collect::<Vec<_>>()
}

fn assigned_slots(
    state: &VirtualGeometryRuntimeState,
    pending_requests: &[VirtualGeometryPageRequest],
) -> BTreeMap<u32, AssignedSlotPlan> {
    let mut assigned_slots = BTreeMap::new();
    let available_slots = available_slots(state);
    let mut next_available_slot = 0usize;
    let mut resident_count = state.resident_slots.len();
    let mut remaining_evictable_pages = state.evictable_pages.clone();

    for request in pending_requests {
        if resident_count < state.page_budget {
            if let Some(slot) = available_slots.get(next_available_slot).copied() {
                assigned_slots.insert(
                    request.page_id,
                    AssignedSlotPlan {
                        slot: Some(slot),
                        recycled_page_id: None,
                    },
                );
                next_available_slot += 1;
                resident_count += 1;
                continue;
            }
        }

        let preferred_recycled_page_id = preferred_recycled_page_id(state, request.page_id);
        let ordered_evictable_pages =
            state.ordered_evictable_pages_for_target(request.page_id, &remaining_evictable_pages);
        let Some(recycled_page_id) = ordered_evictable_pages
            .into_iter()
            .find(|page_id| remaining_evictable_pages.contains(page_id))
        else {
            if let Some(recycled_page_id) = preferred_recycled_page_id {
                assigned_slots.insert(
                    request.page_id,
                    AssignedSlotPlan {
                        slot: None,
                        recycled_page_id: Some(recycled_page_id),
                    },
                );
            }
            continue;
        };
        let Some(recycled_slot) = state.resident_slots.get(&recycled_page_id).copied() else {
            continue;
        };
        assigned_slots.insert(
            request.page_id,
            AssignedSlotPlan {
                slot: Some(recycled_slot),
                recycled_page_id: Some(recycled_page_id),
            },
        );
        remaining_evictable_pages.retain(|page_id| *page_id != recycled_page_id);
    }

    assigned_slots
}

fn current_request_rank(state: &VirtualGeometryRuntimeState, page_id: u32) -> usize {
    state
        .current_requested_page_order
        .get(&page_id)
        .copied()
        .unwrap_or(usize::MAX)
}

fn preferred_recycled_page_id(state: &VirtualGeometryRuntimeState, page_id: u32) -> Option<u32> {
    state
        .ordered_evictable_pages_for_target(page_id, &state.evictable_pages)
        .into_iter()
        .next()
}

fn has_pending_ancestor_request(state: &VirtualGeometryRuntimeState, page_id: u32) -> bool {
    let mut current_page_id = page_id;

    while let Some(parent_page_id) = state.page_parent_pages.get(&current_page_id).copied() {
        if state.pending_pages.contains(&parent_page_id) {
            return true;
        }
        current_page_id = parent_page_id;
    }

    false
}

fn resident_descendant_count(state: &VirtualGeometryRuntimeState, page_id: u32) -> usize {
    descendant_page_ids(state, page_id)
        .into_iter()
        .filter(|descendant_page_id| state.resident_slots.contains_key(descendant_page_id))
        .count()
}

fn descendant_count(state: &VirtualGeometryRuntimeState, page_id: u32) -> usize {
    descendant_page_ids(state, page_id).len()
}

fn descendant_page_ids(state: &VirtualGeometryRuntimeState, page_id: u32) -> Vec<u32> {
    let mut stack = state
        .page_parent_pages
        .iter()
        .filter_map(|(&candidate_page_id, &parent_page_id)| {
            (parent_page_id == page_id).then_some(candidate_page_id)
        })
        .collect::<Vec<_>>();
    let mut descendants = Vec::new();

    while let Some(candidate_page_id) = stack.pop() {
        if descendants.contains(&candidate_page_id) {
            continue;
        }
        descendants.push(candidate_page_id);
        stack.extend(state.page_parent_pages.iter().filter_map(
            |(&grandchild_page_id, &parent_page_id)| {
                (parent_page_id == candidate_page_id).then_some(grandchild_page_id)
            },
        ));
    }

    descendants
}

fn page_depth(state: &VirtualGeometryRuntimeState, page_id: u32) -> usize {
    let mut depth = 0usize;
    let mut current_page_id = page_id;

    while let Some(parent_page_id) = state.page_parent_pages.get(&current_page_id).copied() {
        depth += 1;
        current_page_id = parent_page_id;
    }

    depth
}
