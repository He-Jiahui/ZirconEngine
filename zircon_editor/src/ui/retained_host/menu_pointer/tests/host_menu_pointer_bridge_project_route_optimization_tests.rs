use super::{popup_item_path, prepare_popup_item_path_scratch};

#[test]
fn optimization_batch_20260913_editor_popup_route_scratch_reuses_capacity() {
    let mut scratch = Vec::with_capacity(32);
    scratch.extend([9, 8, 7]);
    let allocation = scratch.as_ptr();

    prepare_popup_item_path_scratch(&mut scratch, 4);

    assert!(scratch.is_empty());
    assert!(scratch.capacity() >= 5);
    assert_eq!(scratch.as_ptr(), allocation);

    prepare_popup_item_path_scratch(&mut scratch, 1);
    assert!(scratch.is_empty());
    assert_eq!(scratch.as_ptr(), allocation);
}

#[test]
fn optimization_batch_20260830cx_popup_item_path_reserves_open_depth_and_hit() {
    let path = popup_item_path(&[2, 4, 1]);

    assert!(path.is_empty());
    assert!(path.capacity() >= 4);
}

#[test]
fn optimization_batch_20260830cx_popup_item_path_capacity_source_contract() {
    let source = include_str!("../host_menu_pointer_bridge_project_route.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("popup route production source");
    assert!(production.contains("prepare_popup_item_path_scratch("));
    assert!(production.contains("item_path: &mut self.popup_item_path_scratch"));
    assert!(!production.contains("popup_item_path("));
    assert!(source.contains("#[cfg(test)]\nfn popup_item_path("));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_20260913_editor_popup_route_scratch_p95() {
    const SAMPLES: usize = 17;
    let open_path = (0..24).collect::<Vec<_>>();

    fn measure_legacy(open_path: &[usize]) -> u128 {
        let started = std::time::Instant::now();
        for _ in 0..16_384 {
            let mut path = popup_item_path(std::hint::black_box(open_path));
            for index in std::hint::black_box(open_path) {
                path.push(*index);
            }
            path.push(open_path.len());
            std::hint::black_box(path);
        }
        started.elapsed().as_nanos().max(1)
    }

    fn measure_reused(open_path: &[usize]) -> u128 {
        let started = std::time::Instant::now();
        let mut path = Vec::new();
        for _ in 0..16_384 {
            prepare_popup_item_path_scratch(&mut path, open_path.len());
            for index in std::hint::black_box(open_path) {
                path.push(*index);
            }
            path.push(open_path.len());
            std::hint::black_box(&path);
        }
        started.elapsed().as_nanos().max(1)
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut reused_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_legacy(&open_path));
            reused_samples.push(measure_reused(&open_path));
        } else {
            reused_samples.push(measure_reused(&open_path));
            legacy_samples.push(measure_legacy(&open_path));
        }
    }
    legacy_samples.sort_unstable();
    reused_samples.sort_unstable();
    let legacy_p95 = legacy_samples[SAMPLES - 1];
    let reused_p95 = reused_samples[SAMPLES - 1];
    println!(
        "EDITOR733_POPUP_ROUTE_SCRATCH_BENCH_V1 depth={} legacy_p95_ns={} reused_p95_ns={} target_ratio_bp=7000",
        open_path.len(),
        legacy_p95,
        reused_p95,
    );
    assert!(
        reused_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(7_000),
        "reused popup route scratch P95 {reused_p95} ns exceeded 70% of legacy {legacy_p95} ns"
    );
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_20260830cx_editor_popup_item_path_capacity_p95() {
    fn measure(open_path: &[usize], reserve: bool) -> u128 {
        let started = std::time::Instant::now();
        for _ in 0..16_384 {
            let mut path = if reserve {
                popup_item_path(open_path)
            } else {
                Vec::new()
            };
            for index in std::hint::black_box(open_path) {
                path.push(*index);
            }
            path.push(open_path.len());
            std::hint::black_box(path);
        }
        started.elapsed().as_nanos()
    }

    let open_path = (0..24).collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(17);
    let mut optimized_samples = Vec::with_capacity(17);
    for sample_index in 0..17 {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure(&open_path, false));
            optimized_samples.push(measure(&open_path, true));
        } else {
            optimized_samples.push(measure(&open_path, true));
            legacy_samples.push(measure(&open_path, false));
        }
    }

    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    let legacy_p95 = legacy_samples[16];
    let optimized_p95 = optimized_samples[16];
    println!(
        "EDITOR340_POPUP_ITEM_PATH_CAPACITY_BENCH_V1 depth={} legacy_p95_ns={} optimized_p95_ns={} target_ratio_bp=7000",
        open_path.len(),
        legacy_p95,
        optimized_p95,
    );
    assert!(
        optimized_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(7_000),
        "preallocated popup path P95 {optimized_p95} ns exceeded 70% of legacy {legacy_p95} ns"
    );
}
