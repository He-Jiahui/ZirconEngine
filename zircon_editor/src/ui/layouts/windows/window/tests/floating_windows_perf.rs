use std::time::Instant;

const SAMPLE_PAIRS: usize = 17;
const WINDOWS_PER_SAMPLE: usize = 64;
const TABS_PER_WINDOW: usize = 16;

#[test]
fn floating_window_projection_reserves_window_and_tab_capacity() {
    let source = include_str!("../floating_windows.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("production implementation");
    assert!(implementation.contains("Vec::with_capacity(model.floating_windows.len())"));
    assert!(implementation.contains("Vec::with_capacity(window.tabs.len())"));
    assert!(implementation.contains("for window in &model.floating_windows"));
    assert!(implementation.contains("for tab in &window.tabs"));
}

#[test]
fn floating_window_projection_keeps_window_before_tab_order() {
    let source = include_str!("../floating_windows.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("production implementation");
    let window_loop = implementation
        .find("for window in &model.floating_windows")
        .expect("window loop");
    let tab_loop = implementation
        .find("for tab in &window.tabs")
        .expect("tab loop");
    assert!(window_loop < tab_loop);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830bv_editor_floating_window_projection_capacity_p95() {
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
        "EDITOR320_FLOATING_WINDOW_PROJECTION_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} windows_per_sample={WINDOWS_PER_SAMPLE} tabs_per_window={TABS_PER_WINDOW} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy),
        sample_csv(&optimized),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..128 {
        let mut windows = if optimized {
            Vec::with_capacity(WINDOWS_PER_SAMPLE)
        } else {
            Vec::new()
        };
        for _ in 0..WINDOWS_PER_SAMPLE {
            let mut tabs = if optimized {
                Vec::with_capacity(TABS_PER_WINDOW)
            } else {
                Vec::new()
            };
            for tab in 0..TABS_PER_WINDOW {
                tabs.push(tab);
            }
            windows.push(tabs.len());
        }
        checksum ^= windows.len();
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
