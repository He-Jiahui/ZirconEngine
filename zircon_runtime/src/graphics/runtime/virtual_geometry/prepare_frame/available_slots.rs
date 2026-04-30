use super::super::VirtualGeometryRuntimeState;

pub(super) fn available_slots(state: &VirtualGeometryRuntimeState) -> Vec<u32> {
    let available_slot_capacity = state
        .page_budget()
        .saturating_sub(state.resident_page_count());
    let mut available_slots = state
        .free_slot_ids()
        .take(available_slot_capacity)
        .collect::<Vec<_>>();
    let future_slot_count = available_slot_capacity.saturating_sub(available_slots.len());
    available_slots
        .extend((0..future_slot_count).map(|index| state.next_slot().saturating_add(index as u32)));
    available_slots
}
