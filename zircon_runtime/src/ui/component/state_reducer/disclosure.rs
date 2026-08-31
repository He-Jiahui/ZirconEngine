use zircon_runtime_interface::ui::component::{UiComponentEventError, UiComponentState, UiValue};

pub(super) fn toggle_expanded(
    state: &mut UiComponentState,
    expanded: bool,
) -> Result<(), UiComponentEventError> {
    state.flags.expanded = expanded;
    super::clear_reference_source(state, "expanded");
    let value = UiValue::Bool(expanded);
    if let Some(existing) = state.values.get_mut("expanded") {
        *existing = value;
    } else {
        state.values.insert("expanded".to_owned(), value);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    #[test]
    fn optimization_batch_eb_existing_expanded_key_updates_in_place() {
        let source = include_str!("disclosure.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("disclosure reducer production implementation");

        assert!(production.contains("state.values.get_mut(\"expanded\")"));
        assert!(production.contains("super::clear_reference_source(state, \"expanded\")"));
        assert!(production.contains("state.values.insert(\"expanded\".to_owned(), value)"));
        assert!(!production.contains("super::set_value"));

        let mut state = UiComponentState::default();
        toggle_expanded(&mut state, true).expect("expand disclosure");
        toggle_expanded(&mut state, false).expect("collapse disclosure");

        assert!(!state.flags.expanded);
        assert_eq!(state.values.get("expanded"), Some(&UiValue::Bool(false)));
    }

    #[test]
    #[ignore = "release-only existing disclosure state key benchmark"]
    fn optimization_batch_eb_existing_disclosure_state_key_release_benchmark_evidence() {
        const SAMPLE_PAIRS: usize = 17;
        const UPDATES_PER_SAMPLE: usize = 65_536;

        fn base_state() -> UiComponentState {
            let mut state = UiComponentState::default();
            state
                .values
                .insert("expanded".to_owned(), UiValue::Bool(false));
            state
        }

        fn legacy_toggle(state: &mut UiComponentState, expanded: bool) {
            state.flags.expanded = expanded;
            super::super::set_value(
                state,
                black_box("expanded").to_owned(),
                UiValue::Bool(expanded),
            );
        }

        fn measure_legacy(base: &UiComponentState) -> u128 {
            let mut state = base.clone();
            let started = Instant::now();
            for update in 0..UPDATES_PER_SAMPLE {
                legacy_toggle(&mut state, black_box(update % 2 == 0));
            }
            black_box(state);
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized(base: &UiComponentState) -> u128 {
            let mut state = base.clone();
            let started = Instant::now();
            for update in 0..UPDATES_PER_SAMPLE {
                toggle_expanded(&mut state, black_box(update % 2 == 0))
                    .expect("valid disclosure toggle");
            }
            black_box(state);
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

        let base = base_state();
        for _ in 0..4 {
            black_box(measure_legacy(&base));
            black_box(measure_optimized(&base));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_legacy(&base));
                optimized_samples.push(measure_optimized(&base));
            } else {
                optimized_samples.push(measure_optimized(&base));
                legacy_samples.push(measure_legacy(&base));
            }
        }

        let legacy_p50_ns = percentile(&legacy_samples, 50);
        let optimized_p50_ns = percentile(&optimized_samples, 50);
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);

        println!(
            "RUNTIME436_EXISTING_DISCLOSURE_STATE_KEY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
updates_per_sample={UPDATES_PER_SAMPLE} pair_order=alternating_legacy_even \
legacy_first_pairs=9 optimized_first_pairs=8 legacy_key_allocations_per_sample={UPDATES_PER_SAMPLE} \
optimized_key_allocations_per_sample=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
            "borrowed existing disclosure key must reduce P95 by at least 30%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }
}
