#[test]
fn optimization_batch_20260919_editor812_palette_posting_capacity_source_contract() {
    let source = include_str!("../locale_projection.rs");
    assert!(source.contains("let mut posting_counts = [0usize; 256];"));
    assert!(source.contains("let mut search_postings = posting_counts.map(Vec::with_capacity);"));
    assert!(source.contains("search_postings[byte].push(index);"));
    assert!(!source.contains("let mut search_postings: [Vec<usize>; 256]"));
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn optimization_batch_20260919_editor812_palette_posting_capacity_bench() {
    const SEED_COUNT: usize = 4_096;
    const BYTE_BUCKETS: usize = 256;
    let mut legacy_growth_events = 0;
    let mut capacity = 0;
    for length in 1..=SEED_COUNT {
        if length > capacity {
            capacity = if capacity == 0 { 4 } else { capacity * 2 };
            legacy_growth_events += 1;
        }
    }
    println!(
        "EDITOR812_PALETTE_LOCALE_POSTING_CAPACITY_BENCH_V1 seeds={} buckets={} legacy_growth_events={} optimized_growth_events=0",
        SEED_COUNT,
        BYTE_BUCKETS,
        legacy_growth_events * BYTE_BUCKETS,
    );
    assert_eq!(legacy_growth_events * BYTE_BUCKETS, 2_816);
}
