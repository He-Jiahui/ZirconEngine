use super::UiHitQueryScratch;
use zircon_runtime_interface::ui::event_ui::UiNodeId;

#[test]
fn historical_high_water_is_released_to_the_current_entry_byte_budget() {
    let mut scratch = UiHitQueryScratch::default();
    scratch.begin(16_384);
    for entry_index in 0..16_384 {
        scratch.insert_candidate(entry_index);
        scratch
            .radius_hits
            .push((0.0, UiNodeId::new(entry_index as u64), entry_index));
    }
    let high_water_bytes = scratch.retained_bytes();

    scratch.begin(32);
    let retained_after_shrink = scratch.retained_bytes();
    let small_entry_budget = UiHitQueryScratch::retained_byte_budget(32);
    let mark_capacity = scratch.marks.capacity();
    let candidate_capacity = scratch.candidates.capacity();

    assert!(retained_after_shrink < high_water_bytes);
    assert!(retained_after_shrink <= small_entry_budget);
    assert!(scratch.candidates.is_empty());
    assert!(scratch.radius_hits.is_empty());

    for entry_index in 0..32 {
        scratch.insert_candidate(entry_index);
        scratch
            .radius_hits
            .push((0.0, UiNodeId::new(entry_index as u64), entry_index));
    }
    scratch.begin(32);

    assert_eq!(scratch.marks.capacity(), mark_capacity);
    assert_eq!(scratch.candidates.capacity(), candidate_capacity);
    assert!(scratch.retained_bytes() <= small_entry_budget);
}

#[test]
fn dedupe_probe_count_scales_linearly_through_ten_thousand_entries() {
    const CELL_REFERENCES_PER_ENTRY: usize = 4;

    for entry_count in [1, 100, 1_000, 10_000] {
        let mut scratch = UiHitQueryScratch::default();
        scratch.begin(entry_count);
        for _ in 0..CELL_REFERENCES_PER_ENTRY {
            for entry_index in 0..entry_count {
                scratch.insert_candidate(entry_index);
            }
        }

        assert_eq!(scratch.candidates.len(), entry_count);
        assert_eq!(
            scratch.dedupe_probes,
            entry_count * CELL_REFERENCES_PER_ENTRY
        );
        assert!(scratch.retained_bytes() <= UiHitQueryScratch::retained_byte_budget(entry_count));
    }
}
