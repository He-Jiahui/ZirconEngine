use std::cmp::Ordering;

use super::summary::RuntimeSessionSlotSummary;

pub(super) fn compare_slot_summary_update_order(
    left: &RuntimeSessionSlotSummary,
    right: &RuntimeSessionSlotSummary,
) -> Ordering {
    left.metadata
        .updated_at_unix_millis
        .unwrap_or(0)
        .cmp(&right.metadata.updated_at_unix_millis.unwrap_or(0))
        .then_with(|| left.slot_id.cmp(&right.slot_id))
}
