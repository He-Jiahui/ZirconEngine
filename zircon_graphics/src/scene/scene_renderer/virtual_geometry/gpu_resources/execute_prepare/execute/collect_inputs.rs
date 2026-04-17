use crate::types::VirtualGeometryPrepareFrame;

use super::super::evictable_slots::evictable_slots;
use super::super::page_table_words::page_table_words;
use super::super::pending_requests::pending_requests;
use super::super::resident_entries::resident_entries;
use super::super::resident_slots::resident_slots;
use super::virtual_geometry_prepare_execution_inputs::VirtualGeometryPrepareExecutionInputs;

pub(super) fn collect_inputs(
    prepare: &VirtualGeometryPrepareFrame,
) -> VirtualGeometryPrepareExecutionInputs {
    let resident_entries = resident_entries(prepare);
    let resident_slots = resident_slots(prepare);
    let pending_requests = pending_requests(prepare);
    let available_slots = prepare.available_slots.clone();
    let evictable_slots = evictable_slots(prepare);
    let page_table_entry_capacity = resident_entries.len() + pending_requests.len();
    let page_table_word_count = page_table_entry_capacity.max(1) * 2;
    let completed_word_count = pending_requests.len().saturating_mul(2) + 1;
    let page_table_words = page_table_words(&resident_entries, page_table_word_count);

    VirtualGeometryPrepareExecutionInputs {
        resident_entries,
        resident_slots,
        pending_requests,
        available_slots,
        evictable_slots,
        page_table_words,
        page_table_word_count,
        completed_word_count,
    }
}
