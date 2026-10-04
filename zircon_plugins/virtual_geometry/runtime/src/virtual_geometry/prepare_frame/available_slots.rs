use super::super::VirtualGeometryRuntimeState;

/// 在预算剩余额度内先复用空槽，再列出尚未分配的槽位编号。
pub(super) fn available_slots(state: &VirtualGeometryRuntimeState) -> Vec<u32> {
    let available_slot_capacity = state
        .page_budget()
        .saturating_sub(state.resident_page_count());
    let mut available_slots = Vec::with_capacity(available_slot_capacity);
    available_slots.extend(state.free_slot_ids().take(available_slot_capacity));
    let future_slot_count = available_slot_capacity.saturating_sub(available_slots.len());
    available_slots
        .extend((0..future_slot_count).map(|index| state.next_slot().saturating_add(index as u32)));
    available_slots
}

#[cfg(test)]
#[path = "available_slots/tests/performance_tests.rs"]
mod performance_tests;
