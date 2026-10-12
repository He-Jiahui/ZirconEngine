use super::*;

#[test]
fn optimization_batch_20260830co_editor_journal_capacity_is_bounded_by_frames_and_limit() {
    const PAYLOAD_OFFSET: usize = 73;

    assert_eq!(journal_entry_capacity(12, PAYLOAD_OFFSET), 0);
    assert_eq!(
        journal_entry_capacity(PAYLOAD_OFFSET + MIN_JOURNAL_FRAME_BYTES * 2, PAYLOAD_OFFSET),
        2
    );
    assert_eq!(
        journal_entry_capacity(
            PAYLOAD_OFFSET + MIN_JOURNAL_FRAME_BYTES * (MAX_JOURNAL_RECORDS + 1),
            PAYLOAD_OFFSET
        ),
        MAX_JOURNAL_RECORDS
    );
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830co_editor_journal_entry_capacity_evidence() {
    let encoded_len = MIN_JOURNAL_FRAME_BYTES * MAX_JOURNAL_RECORDS;
    let legacy_growth_events = collect_growth_events(0);
    let optimized_growth_events = collect_growth_events(journal_entry_capacity(encoded_len, 0));

    println!(
        "EDITOR502_JOURNAL_READ_ENTRY_CAPACITY_BENCH_V1 records={MAX_JOURNAL_RECORDS} \
legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events} \
growth_event_reduction_pct=100"
    );
    assert!(legacy_growth_events > 0);
    assert_eq!(optimized_growth_events, 0);
}

fn collect_growth_events(capacity: usize) -> usize {
    let mut entries = Vec::with_capacity(capacity);
    let mut growth_events = 0;
    for entry in 0..MAX_JOURNAL_RECORDS {
        let previous_capacity = entries.capacity();
        entries.push(entry);
        growth_events += usize::from(entries.capacity() != previous_capacity);
    }
    std::hint::black_box(entries);
    growth_events
}
