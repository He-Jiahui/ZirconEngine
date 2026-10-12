const EDITOR849_LOGICAL_PAINT_CHUNK_CAPACITY_BENCH_V1: &str =
    "EDITOR849_LOGICAL_PAINT_CHUNK_CAPACITY_BENCH_V1";

fn growth_events(item_count: usize, reserved: usize) -> usize {
    let mut capacity = reserved;
    let mut length = 0;
    let mut growths = 0;
    for _ in 0..item_count {
        if length == capacity {
            capacity = capacity.saturating_mul(2).max(4);
            growths += 1;
        }
        length += 1;
    }
    growths
}

#[test]
fn logical_paint_chunk_capacity_matches_source_chunk_length() {
    let chunk_length = 64;
    assert_eq!(growth_events(chunk_length, chunk_length), 0);
    assert_eq!(growth_events(0, 0), 0);
}

#[test]
fn logical_paint_chunk_capacity_model_removes_geometric_growth() {
    let chunk_length = 64;
    assert_eq!(growth_events(chunk_length, 0), 5);
    assert_eq!(growth_events(chunk_length, chunk_length), 0);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn editor849_logical_paint_chunk_capacity_release_benchmark_marker() {
    std::hint::black_box(EDITOR849_LOGICAL_PAINT_CHUNK_CAPACITY_BENCH_V1);
}
