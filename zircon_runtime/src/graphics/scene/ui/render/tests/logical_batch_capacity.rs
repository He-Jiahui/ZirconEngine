const BENCHMARK_MARKER: &str = "RUNTIME859_LOGICAL_TEXT_BATCH_CAPACITY_BENCH_V1";

#[test]
fn runtime859_logical_text_batch_capacity_preserves_order() {
    let lines = [10_u32, 20, 30, 40];
    let (batches, growth_events) = model_batches(lines.iter().copied());

    assert_eq!(batches, lines);
    assert_eq!(growth_events, 0);
}

#[test]
fn runtime859_logical_text_batch_capacity_keeps_empty_layout_zero_capacity() {
    let (batches, growth_events) = model_batches(std::iter::empty());

    assert!(batches.is_empty());
    assert_eq!(growth_events, 0);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime859_logical_text_batch_capacity_bench() {
    let (batches, growth_events) = model_batches(0_u32..4_096);

    println!(
        "{BENCHMARK_MARKER} lines={} growth_events={} reserved_capacity={}",
        batches.len(),
        growth_events,
        batches.capacity()
    );
    assert_eq!(growth_events, 0);
}

fn model_batches<I>(lines: I) -> (Vec<u32>, usize)
where
    I: IntoIterator<Item = u32>,
{
    let lines = lines.into_iter().collect::<Vec<_>>();
    let mut batches = Vec::with_capacity(lines.len());
    let mut growth_events = 0;
    for line in lines {
        if batches.len() == batches.capacity() {
            growth_events += 1;
        }
        batches.push(line);
    }
    (batches, growth_events)
}
