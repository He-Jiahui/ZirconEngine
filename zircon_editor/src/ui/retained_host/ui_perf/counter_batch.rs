use super::UiPerfCounter;

pub(crate) fn record_current_ui_perf_counter_batch(
    collect: impl FnOnce(&mut Vec<(UiPerfCounter, f64)>),
) {
    #[cfg(feature = "profiling")]
    {
        if !cfg!(feature = "profiling-tracy")
            && !zircon_runtime::core::diagnostics::profiling::capture_active()
        {
            return;
        }
        let mut counters = Vec::with_capacity(52);
        collect(&mut counters);
        let scenario = super::current_ui_perf_scenario();
        let named = named_counter_batch(scenario, counters);
        zircon_runtime::core::diagnostics::profiling::record_counter_batch("editor", &named);
    }
    #[cfg(not(feature = "profiling"))]
    {
        let _ = collect;
    }
}

#[cfg(feature = "profiling")]
fn named_counter_batch(
    scenario: super::UiPerfScenario,
    counters: Vec<(UiPerfCounter, f64)>,
) -> Vec<(&'static str, f64)> {
    counters
        .into_iter()
        .map(|(counter, value)| (super::counter_name(scenario, counter), value))
        .collect()
}

#[cfg(all(test, feature = "profiling"))]
#[path = "tests/counter_batch.rs"]
mod tests;
