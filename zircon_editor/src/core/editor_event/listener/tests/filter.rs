use serde_json::json;

#[test]
fn listener_acceptance_does_not_normalize_prefixes_per_record() {
    let source = include_str!("../filter.rs");
    let hot_normalization = [
        "operation_id.starts_with(&",
        "normalize_operation_path_prefix(prefix))",
    ]
    .concat();
    assert!(!source.contains(&hot_normalization));
}

#[test]
fn listener_filter_normalizes_operation_prefixes_once() {
    let filter = super::EditorEventListenerFilter::operation_prefix("  Scene.Node  ");
    assert_eq!(filter.operation_path_prefixes, vec!["scene.node"]);
}

#[test]
fn optimization_wave_20260825vw_editor49_listener_filter_is_compiled_once() {
    let filter: super::EditorEventListenerFilter = serde_json::from_value(json!({
        "operation_path_prefixes": ["  Scene.Node  ", "asset", "scene.node"],
        "operation_groups": ["zeta", "alpha", "middle", "alpha"],
        "sources": ["Headless", "Cli", "Headless"]
    }))
    .expect("listener filter should deserialize");
    let filter = filter.normalized();

    assert_eq!(filter.operation_path_prefixes, vec!["asset", "scene.node"]);
    assert_eq!(
        filter
            .operation_groups()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec!["alpha", "middle", "zeta"]
    );
    assert_eq!(
        filter.sources,
        vec![
            super::EditorEventSource::Headless,
            super::EditorEventSource::Cli
        ]
    );
    assert!(filter.accepts_operation_group("middle"));
    assert!(!filter.accepts_operation_group("missing"));
}

#[test]
fn listener_acceptance_does_not_linearly_scan_operation_groups() {
    let source = include_str!("../filter.rs");
    let linear_group_scan = [".operation_groups", "\n                .iter()"].concat();
    assert!(!source.contains(&linear_group_scan));
}

#[test]
#[ignore = "performance evidence; run in the managed Windows release lane"]
fn optimization_wave_20260825vw_editor49_listener_group_lookup_evidence() {
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    const GROUP_COUNT: usize = 10_000;
    const QUERY_COUNT: usize = 100_000;
    const MAX_ELAPSED: Duration = Duration::from_millis(500);

    let groups = (0..GROUP_COUNT)
        .rev()
        .map(|index| format!("group-{index:05}"))
        .collect::<Vec<_>>();
    let filter: super::EditorEventListenerFilter = serde_json::from_value(json!({
        "operation_groups": groups
    }))
    .expect("listener filter should deserialize");
    let filter = filter.normalized();
    let target = format!("group-{:05}", GROUP_COUNT - 1);

    let started = Instant::now();
    for _ in 0..QUERY_COUNT {
        assert!(black_box(
            filter.accepts_operation_group(black_box(target.as_str()))
        ));
    }
    let elapsed = started.elapsed();

    let legacy_group_comparisons = GROUP_COUNT * QUERY_COUNT;
    let comparisons_per_query_upper_bound =
        usize::BITS as usize - GROUP_COUNT.leading_zeros() as usize;
    let indexed_comparisons_upper_bound = comparisons_per_query_upper_bound * QUERY_COUNT;
    let reduction_basis_points = (legacy_group_comparisons - indexed_comparisons_upper_bound)
        * 10_000
        / legacy_group_comparisons;
    assert!(elapsed <= MAX_ELAPSED, "indexed lookup took {elapsed:?}");
    println!(
        "EDITOR49_LISTENER_FILTER_BENCH_V1 groups={} queries={} legacy_group_comparisons={} indexed_comparisons_upper_bound={} reduction_basis_points={} elapsed_ns={}",
        GROUP_COUNT,
        QUERY_COUNT,
        legacy_group_comparisons,
        indexed_comparisons_upper_bound,
        reduction_basis_points,
        elapsed.as_nanos()
    );
}
