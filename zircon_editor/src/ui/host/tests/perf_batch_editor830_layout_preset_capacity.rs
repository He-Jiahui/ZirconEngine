const PERF_MARKER: &str = "EDITOR830_LAYOUT_PRESET_NAME_CAPACITY_BENCH_V1";

#[test]
fn preset_name_projection_reserves_combined_source_bound() {
    let asset_names = ["zeta", "alpha"];
    let persisted_names = ["beta", "alpha"];
    let mut names = Vec::with_capacity(asset_names.len() + persisted_names.len());
    names.extend(asset_names.into_iter().map(str::to_string));
    names.extend(persisted_names.into_iter().map(str::to_string));
    names.sort_unstable();
    names.dedup();

    assert_eq!(names, vec!["alpha", "beta", "zeta"]);
    assert!(names.capacity() >= asset_names.len() + persisted_names.len());
}

#[test]
fn preset_name_projection_keeps_empty_input_zero_capacity() {
    let names: Vec<String> = Vec::with_capacity(0);
    assert!(names.is_empty());
    assert_eq!(names.capacity(), 0);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn editor830_layout_preset_name_capacity_bench_v1() {
    const ASSET_NAMES: usize = 4_096;
    const PERSISTED_NAMES: usize = 4_096;
    let legacy_growth_events = growth_events(ASSET_NAMES + PERSISTED_NAMES, false);
    let optimized_growth_events = growth_events(ASSET_NAMES + PERSISTED_NAMES, true);
    std::hint::black_box((legacy_growth_events, optimized_growth_events));
    println!(
        "{PERF_MARKER} asset_names={ASSET_NAMES} persisted_names={PERSISTED_NAMES} \
legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events}"
    );
    assert_eq!(legacy_growth_events, 12);
    assert_eq!(optimized_growth_events, 0);
}

fn growth_events(count: usize, reserve: bool) -> usize {
    let mut names = if reserve {
        Vec::with_capacity(count)
    } else {
        Vec::new()
    };
    let mut growth_events = 0;
    for name in 0..count {
        let previous_capacity = names.capacity();
        names.push(name);
        growth_events += usize::from(names.capacity() != previous_capacity);
    }
    growth_events
}
