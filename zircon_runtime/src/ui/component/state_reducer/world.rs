use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiComponentEventError, UiComponentState, UiValidationState, UiValue,
};

pub(super) fn apply_world_transform(
    state: &mut UiComponentState,
    position: [f64; 3],
    rotation: [f64; 3],
    scale: [f64; 3],
) -> Result<(), UiComponentEventError> {
    if scale.iter().any(|value| *value <= 0.0) {
        state.validation = UiValidationState::error("world scale must be positive".to_string());
        return Err(UiComponentEventError::InvalidComplexValue {
            property: "world_scale".to_string(),
            value: format!("{scale:?}"),
        });
    }
    set_world_value(state, "world_position", UiValue::Vec3(position));
    set_world_value(state, "world_rotation", UiValue::Vec3(rotation));
    set_world_value(state, "world_scale", UiValue::Vec3(scale));
    Ok(())
}

pub(super) fn apply_world_surface(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    size: [f64; 2],
    pixels_per_meter: f64,
    billboard: bool,
    depth_test: bool,
    render_order: i64,
    camera_target: String,
) -> Result<(), UiComponentEventError> {
    if size.iter().any(|value| *value <= 0.0) {
        state.validation = UiValidationState::error("world size must be positive".to_string());
        return Err(UiComponentEventError::InvalidComplexValue {
            property: "world_size".to_string(),
            value: format!("{size:?}"),
        });
    }
    let pixels_per_meter = descriptor
        .prop("pixels_per_meter")
        .map(|schema| super::clamp_numeric(pixels_per_meter, schema.min, schema.max))
        .unwrap_or(pixels_per_meter);
    set_world_value(state, "world_size", UiValue::Vec2(size));
    set_world_value(state, "pixels_per_meter", UiValue::Float(pixels_per_meter));
    set_world_value(state, "billboard", UiValue::Bool(billboard));
    set_world_value(state, "depth_test", UiValue::Bool(depth_test));
    set_world_value(state, "render_order", UiValue::Int(render_order));
    set_world_value(state, "camera_target", UiValue::String(camera_target));
    Ok(())
}

fn set_world_value(state: &mut UiComponentState, property: &'static str, value: UiValue) {
    super::clear_reference_source(state, property);
    if let Some(existing) = state.values.get_mut(property) {
        *existing = value;
    } else {
        state.values.insert(property.to_owned(), value);
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    #[test]
    fn optimization_batch_ea_existing_world_keys_update_in_place() {
        let source = include_str!("world.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("world state reducer production implementation");

        assert!(production.contains("if let Some(existing) = state.values.get_mut(property)"));
        assert!(production.contains("super::clear_reference_source(state, property)"));
        assert!(production.contains("state.values.insert(property.to_owned(), value)"));
        assert_eq!(production.matches("set_world_value(").count(), 10);
        assert!(!production.contains("super::set_value"));

        let mut state = UiComponentState::default();
        apply_world_transform(
            &mut state,
            [1.0, 2.0, 3.0],
            [4.0, 5.0, 6.0],
            [1.0, 1.0, 1.0],
        )
        .expect("first world transform");
        apply_world_transform(
            &mut state,
            [7.0, 8.0, 9.0],
            [10.0, 11.0, 12.0],
            [2.0, 2.0, 2.0],
        )
        .expect("updated world transform");

        assert_eq!(
            state.values.get("world_position"),
            Some(&UiValue::Vec3([7.0, 8.0, 9.0]))
        );
        assert_eq!(
            state.values.get("world_rotation"),
            Some(&UiValue::Vec3([10.0, 11.0, 12.0]))
        );
        assert_eq!(
            state.values.get("world_scale"),
            Some(&UiValue::Vec3([2.0, 2.0, 2.0]))
        );
    }

    #[test]
    #[ignore = "release-only existing world state key benchmark"]
    fn optimization_batch_ea_existing_world_state_key_release_benchmark_evidence() {
        const SAMPLE_PAIRS: usize = 17;
        const UPDATES_PER_SAMPLE: usize = 65_536;
        const PROPERTY: &str = "world_position";

        fn base_state() -> UiComponentState {
            let mut state = UiComponentState::default();
            state
                .values
                .insert(PROPERTY.to_owned(), UiValue::Vec3([0.0; 3]));
            state
        }

        fn legacy_set_world_value(state: &mut UiComponentState, property: &str, value: UiValue) {
            super::super::clear_reference_source(state, property);
            state.values.insert(property.to_owned(), value);
        }

        fn measure_legacy(base: &UiComponentState) -> u128 {
            let mut state = base.clone();
            let started = Instant::now();
            for update in 0..UPDATES_PER_SAMPLE {
                legacy_set_world_value(
                    &mut state,
                    black_box(PROPERTY),
                    UiValue::Vec3([update as f64, 0.0, 0.0]),
                );
            }
            black_box(state);
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized(base: &UiComponentState) -> u128 {
            let mut state = base.clone();
            let started = Instant::now();
            for update in 0..UPDATES_PER_SAMPLE {
                set_world_value(
                    &mut state,
                    black_box(PROPERTY),
                    UiValue::Vec3([update as f64, 0.0, 0.0]),
                );
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
            "RUNTIME435_EXISTING_WORLD_STATE_KEY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
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
            "borrowed existing world-state keys must reduce P95 by at least 30%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }
}
