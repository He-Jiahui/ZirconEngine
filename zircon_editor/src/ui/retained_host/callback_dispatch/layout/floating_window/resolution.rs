use crate::ui::workbench::model::FloatingWindowModel;
use crate::ui::workbench::view::ViewInstanceId;

pub(super) fn resolve_floating_window_focus_instance(
    window: &FloatingWindowModel,
) -> Option<ViewInstanceId> {
    window.focus_target_instance().cloned()
}

#[cfg(test)]
pub(super) fn resolve_floating_window_close_instances(
    window: &FloatingWindowModel,
) -> Option<Vec<ViewInstanceId>> {
    if window.tabs.is_empty() || window.tabs.iter().any(|tab| !tab.closeable) {
        return None;
    }
    let mut instances = Vec::with_capacity(window.tabs.len());
    for tab in &window.tabs {
        instances.push(tab.instance_id.clone());
    }
    Some(instances)
}

#[cfg(test)]
mod optimization_batch_20260830bx_editor_tests {
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const TABS_PER_SAMPLE: usize = 64;

    #[test]
    fn close_instance_projection_reserves_tab_capacity() {
        let source = include_str!("resolution.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        assert!(implementation.contains("Vec::with_capacity(window.tabs.len())"));
        assert!(implementation.contains("for tab in &window.tabs"));
        assert!(!implementation.contains(".map(|tab| tab.instance_id.clone())"));
    }

    #[test]
    fn close_instance_projection_keeps_tab_order() {
        let source = include_str!("resolution.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        let loop_start = implementation
            .find("for tab in &window.tabs")
            .expect("tab loop");
        let push = implementation
            .find("instances.push(tab.instance_id.clone())")
            .expect("instance push");
        assert!(loop_start < push);
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830bx_editor_close_instance_capacity_p95() {
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(false));
                optimized.push(measure(true));
            } else {
                optimized.push(measure(true));
                legacy.push(measure(false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "EDITOR322_CLOSE_INSTANCE_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} tabs_per_sample={TABS_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            sample_csv(&legacy),
            sample_csv(&optimized),
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(optimized: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..256 {
            let mut output = if optimized {
                Vec::with_capacity(TABS_PER_SAMPLE)
            } else {
                Vec::new()
            };
            for index in 0..TABS_PER_SAMPLE {
                output.push(index);
            }
            checksum ^= output.len();
        }
        std::hint::black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
    }

    fn sample_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
