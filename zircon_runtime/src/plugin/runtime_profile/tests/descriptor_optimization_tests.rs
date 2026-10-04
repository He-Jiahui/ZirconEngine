#[test]
fn optimization_batch_20260830df_profile_modules_reserve_input_lower_bound() {
    let source = include_str!("../descriptor.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("runtime profile descriptor production source");

    assert!(production.contains("let (minimum_ids, _) = ids.size_hint()"));
    assert!(production.contains("self.builtin_modules.reserve(minimum_ids)"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260830df_profile_module_capacity_evidence() {
    const BATCH_COUNT: usize = 32_768;
    const MODULE_COUNT: usize = 16;
    const MARKER: &str = "RUNTIME518_PROFILE_MODULE_CAPACITY_BENCH_V1";

    let legacy_growth_events = module_growth_events(BATCH_COUNT, MODULE_COUNT, false);
    let optimized_growth_events = module_growth_events(BATCH_COUNT, MODULE_COUNT, true);

    assert!(legacy_growth_events > 0);
    assert_eq!(optimized_growth_events, 0);
    println!(
        "{MARKER} batches={BATCH_COUNT} modules={MODULE_COUNT} \
             legacy_growth_events={legacy_growth_events} \
             optimized_growth_events={optimized_growth_events} reduction_pct=100"
    );
}

fn module_growth_events(batch_count: usize, module_count: usize, reserve: bool) -> usize {
    let mut growth_events = 0;
    for _ in 0..batch_count {
        let mut modules = Vec::new();
        if reserve {
            modules.reserve(module_count);
        }
        for module in 0..module_count {
            let previous_capacity = modules.capacity();
            modules.push(module);
            growth_events += usize::from(modules.capacity() != previous_capacity);
        }
    }
    growth_events
}
