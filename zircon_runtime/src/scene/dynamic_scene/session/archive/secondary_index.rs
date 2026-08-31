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
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::{
        SlotUpdateIndex, TagSlotIndices, index_secondary_entries, remove_secondary_entries,
    };

    const SCENE_PAYLOAD_BYTES: usize = 1024 * 1024;
    const TAG_COUNT: usize = 64;
    const UPDATES_PER_SAMPLE: usize = 32;
    const SAMPLE_PAIRS: usize = 21;

    #[derive(Clone)]
    struct BenchmarkSlot {
        slot_id: String,
        updated_at: u64,
        tags: Vec<String>,
        scene_payload: Vec<u8>,
    }

    impl BenchmarkSlot {
        fn new(tag_prefix: &str) -> Self {
            Self {
                slot_id: "manual.large-scene".to_string(),
                updated_at: 1,
                tags: tags(tag_prefix),
                scene_payload: vec![0x5a; SCENE_PAYLOAD_BYTES],
            }
        }

        fn update_key(&self) -> (u64, String) {
            (self.updated_at, self.slot_id.clone())
        }
    }

    fn tags(prefix: &str) -> Vec<String> {
        (0..TAG_COUNT)
            .map(|index| format!("{prefix}.{index:03}"))
            .collect()
    }

    fn initialized_indices(slot: &BenchmarkSlot) -> (SlotUpdateIndex, TagSlotIndices) {
        let mut updated = SlotUpdateIndex::new();
        let mut tagged = TagSlotIndices::new();
        index_secondary_entries(&mut updated, &mut tagged, 0, slot.update_key(), &slot.tags);
        (updated, tagged)
    }

    #[test]
    fn runtime52_secondary_index_preserves_ordered_queries() {
        let mut updated = SlotUpdateIndex::new();
        let mut tagged = TagSlotIndices::new();
        let manual = vec!["manual".to_string(), "reviewed".to_string()];
        let autosave = vec!["autosave".to_string()];

        index_secondary_entries(
            &mut updated,
            &mut tagged,
            7,
            (20, "manual".to_string()),
            &manual,
        );
        index_secondary_entries(
            &mut updated,
            &mut tagged,
            3,
            (10, "autosave".to_string()),
            &autosave,
        );

        assert_eq!(updated.values().copied().collect::<Vec<_>>(), vec![3, 7]);
        assert_eq!(
            tagged["reviewed"].values().copied().collect::<Vec<_>>(),
            vec![7]
        );

        remove_secondary_entries(
            &mut updated,
            &mut tagged,
            &(20, "manual".to_string()),
            &manual,
        );
        assert_eq!(updated.values().copied().collect::<Vec<_>>(), vec![3]);
        assert!(!tagged.contains_key("manual"));
        assert!(!tagged.contains_key("reviewed"));
    }

    fn legacy_update(
        slot: &mut BenchmarkSlot,
        updated: &mut SlotUpdateIndex,
        tagged: &mut TagSlotIndices,
        replacement_tags: Vec<String>,
    ) -> Vec<String> {
        let previous = slot.clone();
        let previous_key = previous.update_key();
        remove_secondary_entries(updated, tagged, &previous_key, &previous.tags);
        slot.updated_at = slot.updated_at.saturating_add(1);
        slot.tags = replacement_tags;
        let update_key = slot.update_key();
        let tags = slot.tags.clone();
        index_secondary_entries(updated, tagged, 0, update_key, &tags);
        let BenchmarkSlot {
            tags: previous_tags,
            scene_payload,
            ..
        } = previous;
        black_box(scene_payload);
        previous_tags
    }

    fn optimized_update(
        slot: &mut BenchmarkSlot,
        updated: &mut SlotUpdateIndex,
        tagged: &mut TagSlotIndices,
        replacement_tags: Vec<String>,
    ) -> Vec<String> {
        let previous_tags = std::mem::replace(&mut slot.tags, replacement_tags);
        let previous_key = slot.update_key();
        remove_secondary_entries(updated, tagged, &previous_key, &previous_tags);
        slot.updated_at = slot.updated_at.saturating_add(1);
        index_secondary_entries(updated, tagged, 0, slot.update_key(), &slot.tags);
        previous_tags
    }

    fn measure_updates(legacy: bool) -> u128 {
        let mut slot = BenchmarkSlot::new("primary");
        let (mut updated, mut tagged) = initialized_indices(&slot);
        let mut replacement_tags = tags("replacement");
        let start = Instant::now();
        for _ in 0..UPDATES_PER_SAMPLE {
            replacement_tags = if legacy {
                legacy_update(&mut slot, &mut updated, &mut tagged, replacement_tags)
            } else {
                optimized_update(&mut slot, &mut updated, &mut tagged, replacement_tags)
            };
        }
        let elapsed = start.elapsed().as_nanos();
        black_box((slot, updated, tagged, replacement_tags));
        elapsed
    }

    fn p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100) - 1]
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn runtime52_metadata_index_evidence() {
        for _ in 0..3 {
            black_box(measure_updates(true));
            black_box(measure_updates(false));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample in 0..SAMPLE_PAIRS {
            if sample % 2 == 0 {
                legacy_samples.push(measure_updates(true));
                optimized_samples.push(measure_updates(false));
            } else {
                optimized_samples.push(measure_updates(false));
                legacy_samples.push(measure_updates(true));
            }
        }
        let legacy_p95_ns = p95(&mut legacy_samples);
        let optimized_p95_ns = p95(&mut optimized_samples);
        let reduction = 100.0 - optimized_p95_ns as f64 * 100.0 / legacy_p95_ns as f64;
        println!(
            "RUNTIME52_METADATA_INDEX_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             updates_per_sample={UPDATES_PER_SAMPLE} scene_payload_bytes={SCENE_PAYLOAD_BYTES} \
             tags={TAG_COUNT} legacy_scene_payload_clones={UPDATES_PER_SAMPLE} \
             optimized_scene_payload_clones=0 legacy_tag_vector_clones={} \
             optimized_tag_vector_clones=0 legacy_p95_ns={legacy_p95_ns} \
             optimized_p95_ns={optimized_p95_ns} p95_reduction_pct={reduction:.3}",
            UPDATES_PER_SAMPLE * 2,
        );
        assert!(
            optimized_p95_ns <= legacy_p95_ns.saturating_mul(50) / 100,
            "borrowed secondary indexing should be <=50% of legacy clone path: \
             optimized={optimized_p95_ns}ns legacy={legacy_p95_ns}ns"
        );
    }
}
