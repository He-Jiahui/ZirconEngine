use crate::virtual_geometry::types::VirtualGeometryPrepareFrame;

use super::super::frame_projections::{
    evictable_slots_and_reclaimable_bytes, resident_entries_and_slots,
};
use super::super::page_table_words::page_table_words;
use super::super::pending_requests::pending_requests;
use super::virtual_geometry_prepare_execution_inputs::VirtualGeometryPrepareExecutionInputs;

pub(super) fn collect_inputs(
    prepare: &VirtualGeometryPrepareFrame,
) -> VirtualGeometryPrepareExecutionInputs {
    let (resident_entries, resident_slots) = resident_entries_and_slots(prepare);
    let pending_requests = pending_requests(prepare);
    let available_slots = prepare.available_slots.clone();
    let (evictable_slots, reclaimable_bytes) = evictable_slots_and_reclaimable_bytes(prepare);
    let page_table_entry_capacity = resident_entries.len() + pending_requests.len();
    let page_table_word_count = page_table_entry_capacity.max(1) * 2;
    let completed_word_count = pending_requests.len().saturating_mul(3) + 1;
    let page_table_words = page_table_words(&resident_entries, page_table_word_count);

    VirtualGeometryPrepareExecutionInputs {
        resident_entries,
        resident_slots,
        pending_requests,
        available_slots,
        evictable_slots,
        reclaimable_bytes,
        page_table_words,
        page_table_word_count,
        completed_word_count,
    }
}
