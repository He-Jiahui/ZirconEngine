const BENCHMARK_MARKER: &str = "EDITOR858_SAVE_BATCH_FAILURE_CAPACITY_BENCH_V1";

#[test]
fn editor858_save_batch_failure_capacity_preserves_order() {
    let failures = model_failures([3_u32, 1, 2]);

    assert_eq!(failures, vec![3, 1, 2]);
}

#[test]
fn editor858_save_batch_failure_capacity_keeps_empty_batch_zero_capacity() {
    let (failures, growth_events) = model_failures_with_growth(std::iter::empty());

    assert!(failures.is_empty());
    assert_eq!(growth_events, 0);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn editor858_save_batch_failure_capacity_bench() {
    let (failures, growth_events) = model_failures_with_growth(0_u32..4_096);

    println!(
        "{BENCHMARK_MARKER} candidates={} failures={} growth_events={} reserved_capacity={}",
        failures.len(),
        failures.len(),
        growth_events,
        failures.capacity()
    );
    assert_eq!(growth_events, 0);
}

fn model_failures<I>(candidates: I) -> Vec<u32>
where
    I: IntoIterator<Item = u32>,
{
    model_failures_with_growth(candidates).0
}

fn model_failures_with_growth<I>(candidates: I) -> (Vec<u32>, usize)
where
    I: IntoIterator<Item = u32>,
{
    let candidates = candidates.into_iter().collect::<Vec<_>>();
    let mut failures = Vec::with_capacity(candidates.len());
    let mut growth_events = 0;
    for candidate in candidates {
        if failures.len() == failures.capacity() {
            growth_events += 1;
        }
        failures.push(candidate);
    }
    (failures, growth_events)
}
