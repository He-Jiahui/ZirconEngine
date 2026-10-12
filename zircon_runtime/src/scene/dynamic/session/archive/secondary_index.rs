use std::collections::BTreeMap;

type SlotUpdateKey = (u64, String);
type SlotUpdateIndex = BTreeMap<SlotUpdateKey, usize>;
type TagSlotIndices = BTreeMap<String, SlotUpdateIndex>;

pub(super) fn index_secondary_entries(
    updated_slot_indices: &mut SlotUpdateIndex,
    tag_slot_indices: &mut TagSlotIndices,
    slot_index: usize,
    update_key: SlotUpdateKey,
    tags: &[String],
) {
    updated_slot_indices.insert(update_key.clone(), slot_index);
    for tag in tags {
        tag_slot_indices
            .entry(tag.clone())
            .or_default()
            .insert(update_key.clone(), slot_index);
    }
}

pub(super) fn remove_secondary_entries(
    updated_slot_indices: &mut SlotUpdateIndex,
    tag_slot_indices: &mut TagSlotIndices,
    update_key: &SlotUpdateKey,
    tags: &[String],
) {
    updated_slot_indices.remove(update_key);
    let mut empty_tags = Vec::new();
    for tag in tags {
        let Some(tag_indices) = tag_slot_indices.get_mut(tag) else {
            continue;
        };
        tag_indices.remove(update_key);
        if tag_indices.is_empty() {
            empty_tags.push(tag.as_str());
        }
    }
    for tag in empty_tags {
        tag_slot_indices.remove(tag);
    }
}

#[cfg(test)]
#[path = "tests/secondary_index.rs"]
mod tests;
