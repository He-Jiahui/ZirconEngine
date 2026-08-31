use crate::ui::binding::EditorUiBinding;
use crate::ui::template_runtime::showcase_demo_state::{
    resolve_showcase_component_event, UiComponentShowcaseDemoError,
    UiComponentShowcaseDemoEventInput, UiComponentShowcaseDemoState,
};
use zircon_runtime_interface::ui::component::{
    UiComponentAdapterResult, UiComponentProjectionPatch, UiValue,
};

pub(crate) fn apply_showcase_component_binding(
    state: &mut UiComponentShowcaseDemoState,
    binding: &EditorUiBinding,
    input: UiComponentShowcaseDemoEventInput,
) -> Result<UiComponentAdapterResult, UiComponentShowcaseDemoError> {
    let resolved = resolve_showcase_component_event(binding, input)?;
    let changed_value = state.apply_component_event_envelope(
        &resolved.action,
        &resolved.envelope,
        resolved.changed_property.as_deref(),
    )?;
    let control_id = resolved.envelope.control_id;
    let changed_property = resolved.changed_property;

    let mut patch = UiComponentProjectionPatch::new(control_id);
    if let (Some(property), Some(value)) = (changed_property, changed_value) {
        let value_text = value.display_text();
        patch = patch
            .with_state_value(property, value)
            .with_attribute("value_text", UiValue::String(value_text));
    }

    Ok(UiComponentAdapterResult::changed().with_patch(patch))
}

#[cfg(test)]
mod performance_tests {
    use std::hint::black_box;
    use std::time::Instant;

    #[derive(Clone)]
    struct ResolvedFixture {
        control_id: String,
        changed_property: Option<String>,
        changed_value: String,
    }

    #[test]
    fn optimization_batch_eh_showcase_patch_moves_resolved_fields_after_dispatch() {
        let source = include_str!("showcase.rs");
        let implementation = source
            .split("fn apply_showcase_component_binding")
            .nth(1)
            .expect("showcase component binding implementation")
            .split("#[cfg(test)]")
            .next()
            .expect("bounded production implementation");
        let dispatch = implementation
            .find("state.apply_component_event_envelope(")
            .expect("component event dispatch");
        let field_move = implementation
            .find("let control_id = resolved.envelope.control_id;")
            .expect("resolved control id move");

        assert!(dispatch < field_move);
        assert!(implementation.contains("let value_text = value.display_text();"));
        assert!(!implementation.contains("control_id.clone()"));
        assert!(!implementation.contains("changed_property.clone()"));
        assert!(!implementation.contains("value.clone()"));
    }

    #[test]
    #[ignore = "release-only direct showcase patch field move benchmark"]
    fn optimization_batch_eh_direct_showcase_patch_field_move_release_benchmark_evidence() {
        const SAMPLE_PAIRS: usize = 17;
        const PROJECTIONS_PER_SAMPLE: usize = 8_192;

        fn measure_legacy(base: &ResolvedFixture) -> u128 {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..PROJECTIONS_PER_SAMPLE {
                let resolved = black_box(base.clone());
                black_box((
                    &resolved.control_id,
                    resolved.changed_property.as_deref(),
                    &resolved.changed_value,
                ));
                let patch = black_box((
                    resolved.control_id.clone(),
                    resolved.changed_property.clone().expect("fixture property"),
                    resolved.changed_value.clone(),
                ));
                checksum = checksum
                    .wrapping_add(patch.0.len())
                    .wrapping_add(patch.1.len())
                    .wrapping_add(patch.2.len());
                black_box((resolved, patch));
            }
            black_box(checksum);
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized(base: &ResolvedFixture) -> u128 {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..PROJECTIONS_PER_SAMPLE {
                let resolved = black_box(base.clone());
                black_box((
                    &resolved.control_id,
                    resolved.changed_property.as_deref(),
                    &resolved.changed_value,
                ));
                let patch = black_box((
                    resolved.control_id,
                    resolved.changed_property.expect("fixture property"),
                    resolved.changed_value,
                ));
                checksum = checksum
                    .wrapping_add(patch.0.len())
                    .wrapping_add(patch.1.len())
                    .wrapping_add(patch.2.len());
                black_box(patch);
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

        let base = ResolvedFixture {
            control_id: format!("Showcase/{}Control", "nested/".repeat(32)),
            changed_property: Some(format!("property.{}value", "segment.".repeat(32))),
            changed_value: format!("{}payload", "component-value-".repeat(32)),
        };
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample in 0..SAMPLE_PAIRS {
            if sample % 2 == 0 {
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
            "EDITOR370_DIRECT_SHOWCASE_PATCH_FIELD_MOVE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             projections_per_sample={PROJECTIONS_PER_SAMPLE} control_bytes={} property_bytes={} value_bytes={} \
             pair_order=alternating_legacy_even legacy_extra_field_clones_per_sample={} \
             optimized_extra_field_clones_per_sample=0 legacy_p50_ns={legacy_p50_ns} \
             optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
             optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            base.control_id.len(),
            base.changed_property.as_deref().expect("fixture property").len(),
            base.changed_value.len(),
            PROJECTIONS_PER_SAMPLE * 3,
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
            "direct showcase patch field moves must reduce P95 by at least 30%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }
}
