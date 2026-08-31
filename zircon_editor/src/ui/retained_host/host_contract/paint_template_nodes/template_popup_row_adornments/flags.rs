use super::super::super::data::TemplatePaneMenuItemData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn menu_item_has_flag(
    item: &TemplatePaneMenuItemData,
    expected: &str,
) -> bool {
    menu_item_flags(item).any(|flag| flag.eq_ignore_ascii_case(expected))
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn menu_item_loading_and_danger(
    item: &TemplatePaneMenuItemData,
) -> (bool, bool) {
    let mut loading = item.loading;
    let mut danger = false;

    for flag in menu_item_flags(item) {
        match flag.len() {
            7 if !loading && flag.eq_ignore_ascii_case("loading") => loading = true,
            6 if !danger && flag.eq_ignore_ascii_case("danger") => danger = true,
            _ => {}
        }
        if loading && danger {
            break;
        }
    }

    (loading, danger)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn menu_item_flag_value<
    'a,
>(
    item: &'a TemplatePaneMenuItemData,
    expected_key: &str,
) -> Option<&'a str> {
    menu_item_flags(item).find_map(|flag| {
        let (key, value) = flag.split_once('=')?;
        let value = value.trim();
        (key.trim().eq_ignore_ascii_case(expected_key) && !value.is_empty()).then_some(value)
    })
}

fn menu_item_flags(item: &TemplatePaneMenuItemData) -> impl Iterator<Item = &str> {
    item.raw
        .as_str()
        .split('|')
        .nth(1)
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|flag| !flag.is_empty())
}

#[cfg(test)]
mod optimization_batch_es_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    fn item(raw: String, loading: bool) -> TemplatePaneMenuItemData {
        TemplatePaneMenuItemData {
            raw,
            loading,
            ..TemplatePaneMenuItemData::default()
        }
    }

    fn legacy_loading_and_danger(item: &TemplatePaneMenuItemData) -> (bool, bool) {
        (
            item.loading || menu_item_has_flag(item, "loading"),
            menu_item_has_flag(item, "danger"),
        )
    }

    #[test]
    fn optimization_batch_es_menu_flags_preserve_loading_and_danger_semantics() {
        for item in [
            item("save|checked,LOADING,Danger".to_string(), false),
            item("save|checked,danger".to_string(), true),
            item("save|loading-state,dangerous".to_string(), false),
            item("save".to_string(), false),
        ] {
            assert_eq!(
                menu_item_loading_and_danger(&item),
                legacy_loading_and_danger(&item)
            );
        }
    }

    #[test]
    #[ignore = "release-only single-pass menu flag benchmark"]
    fn optimization_batch_es_single_pass_menu_flags_release_benchmark_evidence() {
        const SAMPLE_PAIRS: usize = 17;
        const CHECKS_PER_SAMPLE: usize = 8_192;
        const FLAG_COUNT: usize = 128;

        fn measure_legacy(item: &TemplatePaneMenuItemData) -> u128 {
            let started = Instant::now();
            let mut checksum = 0_usize;
            for _ in 0..CHECKS_PER_SAMPLE {
                let (loading, danger) = legacy_loading_and_danger(black_box(item));
                checksum = checksum.wrapping_add(usize::from(loading) + usize::from(danger));
            }
            black_box(checksum);
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized(item: &TemplatePaneMenuItemData) -> u128 {
            let started = Instant::now();
            let mut checksum = 0_usize;
            for _ in 0..CHECKS_PER_SAMPLE {
                let (loading, danger) = menu_item_loading_and_danger(black_box(item));
                checksum = checksum.wrapping_add(usize::from(loading) + usize::from(danger));
            }
            black_box(checksum);
            started.elapsed().as_nanos().max(1)
        }

        fn percentile(samples: &[u128], percentile: usize) -> u128 {
            let mut sorted = samples.to_vec();
            sorted.sort_unstable();
            let rank = (sorted.len() * percentile).div_ceil(100);
            sorted[rank.saturating_sub(1)]
        }

        fn raw(samples: &[u128]) -> String {
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        }

        let mut flags = (0..FLAG_COUNT - 2)
            .map(|index| format!("option_{index:03}"))
            .collect::<Vec<_>>();
        flags.push("loading".to_string());
        flags.push("danger".to_string());
        let item = item(format!("save|{}", flags.join(",")), false);
        assert_eq!(
            menu_item_loading_and_danger(&item),
            legacy_loading_and_danger(&item)
        );

        for _ in 0..4 {
            black_box(measure_legacy(&item));
            black_box(measure_optimized(&item));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample in 0..SAMPLE_PAIRS {
            if sample % 2 == 0 {
                legacy_samples.push(measure_legacy(&item));
                optimized_samples.push(measure_optimized(&item));
            } else {
                optimized_samples.push(measure_optimized(&item));
                legacy_samples.push(measure_legacy(&item));
            }
        }

        let legacy_p50_ns = percentile(&legacy_samples, 50);
        let optimized_p50_ns = percentile(&optimized_samples, 50);
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);
        println!(
            "EDITOR381_SINGLE_PASS_MENU_FLAGS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             checks_per_sample={CHECKS_PER_SAMPLE} flag_count={FLAG_COUNT} \
             pair_order=alternating_legacy_even legacy_flag_passes_per_check=2 \
             optimized_flag_passes_per_check=1 legacy_p50_ns={legacy_p50_ns} \
             optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
             optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(60),
            "single-pass menu flag parsing must reduce P95 by at least 40%: \
             legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }
}
