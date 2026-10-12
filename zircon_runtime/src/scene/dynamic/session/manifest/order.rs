use std::cmp::Ordering;

use super::summary::RuntimeSessionSlotSummary;

// 未标记更新时间的槽位按零排序；时间并列时以槽位 ID 决定结果，使目录选择与负载索引一致。
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
