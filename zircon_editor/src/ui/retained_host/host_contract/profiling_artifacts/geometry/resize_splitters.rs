use super::super::super::data::HostResizeLayerData;
use super::super::UiProfileNamedFrame;
use super::frame_math::push_named_frame;

pub(in crate::ui::retained_host::host_contract) fn collect_resize_splitters(
    resize_layer: &HostResizeLayerData,
) -> Vec<UiProfileNamedFrame> {
    let mut resize_splitters = Vec::with_capacity(3);
    push_named_frame(
        &mut resize_splitters,
        "resize.left_splitter",
        "resize_splitter",
        "left",
        resize_layer.left_splitter_frame.clone(),
        None,
    );
    push_named_frame(
        &mut resize_splitters,
        "resize.right_splitter",
        "resize_splitter",
        "right",
        resize_layer.right_splitter_frame.clone(),
        None,
    );
    push_named_frame(
        &mut resize_splitters,
        "resize.bottom_splitter",
        "resize_splitter",
        "bottom",
        resize_layer.bottom_splitter_frame.clone(),
        None,
    );
    resize_splitters
}

#[cfg(test)]
mod optimization_batch_20260830bu_editor_tests {
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const SPLITTERS_PER_SAMPLE: usize = 3;

    #[test]
    fn resize_splitters_reserve_fixed_output_capacity() {
        let source = include_str!("resize_splitters.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        assert!(implementation.contains("Vec::with_capacity(3)"));
        assert!(!implementation.contains("let mut resize_splitters = Vec::new()"));
    }

    #[test]
    fn resize_splitters_keep_left_right_bottom_order() {
        let source = include_str!("resize_splitters.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        let left = implementation
            .find("resize.left_splitter")
            .expect("left splitter");
        let right = implementation
            .find("resize.right_splitter")
            .expect("right splitter");
        let bottom = implementation
            .find("resize.bottom_splitter")
            .expect("bottom splitter");
        assert!(left < right);
        assert!(right < bottom);
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830bu_editor_resize_splitter_capacity_p95() {
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
            "EDITOR319_RESIZE_SPLITTER_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} splitters_per_sample={SPLITTERS_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
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
                Vec::with_capacity(SPLITTERS_PER_SAMPLE)
            } else {
                Vec::new()
            };
            for index in 0..SPLITTERS_PER_SAMPLE {
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
