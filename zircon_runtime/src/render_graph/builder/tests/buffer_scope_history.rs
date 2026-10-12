use super::*;

#[test]
fn neighbor_coalescing_preserves_whole_map_history_after_each_update() {
    let mut optimized = BufferScopeHistory::new(16);
    let mut legacy = optimized.clone();
    let mut optimized_work = AccessScopeWorkReceipt::default();
    let mut legacy_work = AccessScopeWorkReceipt::default();
    for (start, end, ordinal) in [
        (4, 8, 1),
        (8, 12, 1),
        (2, 6, 2),
        (6, 14, 2),
        (0, 16, 3),
        (1, 15, 4),
        (0, 1, 4),
        (15, 16, 4),
    ] {
        for (history, work, whole_map) in [
            (&mut optimized, &mut optimized_work, false),
            (&mut legacy, &mut legacy_work, true),
        ] {
            history.ensure_boundaries(start, end, 0, work).unwrap();
            for (_, segment) in history.segments.range_mut(start..end) {
                segment.history.latest_version_ordinal = ordinal;
            }
            if whole_map {
                with_whole_map_coalescing(|| history.merge_adjacent_equal_around(start, end, work));
            } else {
                history.merge_adjacent_equal_around(start, end, work);
            }
        }
        assert_eq!(legacy.segments, optimized.segments, "range {start}..{end}");
    }
    assert_eq!(optimized.segments.len(), 1);
}

#[test]
fn neighbor_coalescing_does_not_visit_unaffected_segments() {
    const SEGMENT_COUNT: u64 = 10_000;
    let mut history = BufferScopeHistory::new(SEGMENT_COUNT);
    let mut work = AccessScopeWorkReceipt::default();
    for boundary in 1..SEGMENT_COUNT {
        history
            .ensure_boundaries(boundary, boundary + 1, 0, &mut work)
            .unwrap();
    }
    for (index, segment) in history.segments.values_mut().enumerate() {
        segment.history.latest_version_ordinal = index as u64;
    }
    let before = work.merge_visits;
    history.merge_adjacent_equal_around(5_000, 5_001, &mut work);

    // The coalescer compares only the segment immediately before the
    // affected range and the one immediately after it. The other 9,998
    // boundaries must not be visited.
    assert_eq!(work.merge_visits - before, 2);
}

#[test]
fn neighbor_coalescing_handles_map_boundaries() {
    let mut history = BufferScopeHistory::new(4);
    let mut work = AccessScopeWorkReceipt::default();
    for boundary in 1..4 {
        history
            .ensure_boundaries(boundary, boundary + 1, 0, &mut work)
            .unwrap();
    }
    let before = work.merge_visits;
    history.merge_adjacent_equal_around(0, 1, &mut work);
    assert_eq!(work.merge_visits - before, 1);

    let before = work.merge_visits;
    history.merge_adjacent_equal_around(3, 4, &mut work);
    assert_eq!(work.merge_visits - before, 1);
}
