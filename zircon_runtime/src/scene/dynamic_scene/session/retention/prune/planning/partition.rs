use std::collections::BTreeSet;

// 仅接收按槽位 ID 递增的完整列表；计划层用规范索引提供该顺序，以线性扫描保持报告顺序。
pub(super) fn partition_pruned_slot_ids(
    all_slot_ids: impl IntoIterator<Item = String>,
    scoped_slot_ids: &BTreeSet<String>,
    kept_slot_ids: &BTreeSet<String>,
) -> (Vec<String>, Vec<String>) {
    let mut scoped_slot_ids = scoped_slot_ids.iter().peekable();
    let mut kept_slot_ids = kept_slot_ids.iter().peekable();
    all_slot_ids.into_iter().partition(|slot_id| {
        while scoped_slot_ids
            .peek()
            .is_some_and(|candidate| candidate.as_str() < slot_id.as_str())
        {
            scoped_slot_ids.next();
        }
        let is_scoped = scoped_slot_ids
            .peek()
            .is_some_and(|candidate| candidate.as_str() == slot_id.as_str());
        if !is_scoped {
            return true;
        }

        while kept_slot_ids
            .peek()
            .is_some_and(|candidate| candidate.as_str() < slot_id.as_str())
        {
            kept_slot_ids.next();
        }
        kept_slot_ids
            .peek()
            .is_some_and(|candidate| candidate.as_str() == slot_id.as_str())
    })
}

#[cfg(test)]
#[path = "tests/partition.rs"]
mod tests;
