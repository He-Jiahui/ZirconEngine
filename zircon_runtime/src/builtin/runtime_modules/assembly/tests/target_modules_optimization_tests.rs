#[test]
fn optimization_batch_20260830dg_target_modules_reserve_manifest_upper_bound() {
    let source = include_str!("../target_modules.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("target module assembly production source");

    assert!(production.contains("report.modules.reserve(manifest.selections.len())"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260830dg_target_module_capacity_evidence() {
    const BATCH_COUNT: usize = 32_768;
    const CORE_MODULE_COUNT: usize = 16;
    const PLUGIN_SELECTION_COUNT: usize = 64;
    const MARKER: &str = "RUNTIME519_TARGET_MODULE_CAPACITY_BENCH_V1";

    let legacy_growth_events = module_growth_events(
        BATCH_COUNT,
        CORE_MODULE_COUNT,
        PLUGIN_SELECTION_COUNT,
        false,
    );
    let optimized_growth_events =
        module_growth_events(BATCH_COUNT, CORE_MODULE_COUNT, PLUGIN_SELECTION_COUNT, true);

    assert!(legacy_growth_events > 0);
    assert_eq!(optimized_growth_events, 0);
    println!(
        "{MARKER} batches={BATCH_COUNT} core_modules={CORE_MODULE_COUNT} \
             plugin_selections={PLUGIN_SELECTION_COUNT} \
             legacy_growth_events={legacy_growth_events} \
             optimized_growth_events={optimized_growth_events} reduction_pct=100"
    );
}

fn module_growth_events(
    batch_count: usize,
    core_module_count: usize,
    plugin_selection_count: usize,
    reserve_plugins: bool,
) -> usize {
    let mut growth_events = 0;
    for _ in 0..batch_count {
        let mut modules = Vec::with_capacity(core_module_count);
        modules.extend(0..core_module_count);
        if reserve_plugins {
            modules.reserve(plugin_selection_count);
        }
        for plugin in 0..plugin_selection_count {
            let previous_capacity = modules.capacity();
            modules.push(core_module_count + plugin);
            growth_events += usize::from(modules.capacity() != previous_capacity);
        }
    }
    growth_events
}
