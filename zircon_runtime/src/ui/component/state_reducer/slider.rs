use zircon_runtime_interface::ui::component::{UiComponentDescriptor, UiComponentState, UiValue};

pub(super) fn sync_after_value_change(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
) {
    if descriptor.role != "range-slider" {
        return;
    }

    match property {
        "value" => sync_percent_from_value(state, descriptor, "value", "value_percent"),
        "range_min" => sync_percent_from_value(state, descriptor, "range_min", "range_min_percent"),
        "value_percent" => sync_value_from_percent(state, descriptor, "value_percent", "value"),
        "range_min_percent" => {
            sync_value_from_percent(state, descriptor, "range_min_percent", "range_min")
        }
        "min" | "max" => {
            sync_percent_from_value(state, descriptor, "range_min", "range_min_percent");
            sync_percent_from_value(state, descriptor, "value", "value_percent");
        }
        _ => {}
    }
}

fn sync_value_from_percent(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    percent_property: &str,
    value_property: &str,
) {
    if descriptor.prop(percent_property).is_none() || descriptor.prop(value_property).is_none() {
        return;
    }

    let Some(percent) = super::numeric_component_value(state, descriptor, percent_property) else {
        return;
    };
    let (min, max) = range_bounds(state, descriptor);
    let raw_value = if (max - min).abs() <= f64::EPSILON {
        min
    } else {
        min + (max - min) * percent.clamp(0.0, 1.0)
    };
    let Some(schema) = descriptor.prop(value_property) else {
        return;
    };
    let value = super::clamp_component_numeric_value(
        state,
        descriptor,
        value_property,
        schema.min,
        schema.max,
        raw_value,
    );
    set_slider_value(
        state,
        value_property,
        super::numeric_value(schema.value_kind, value),
    );
    sync_percent_from_value(state, descriptor, value_property, percent_property);
}

fn sync_percent_from_value(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    value_property: &str,
    percent_property: &str,
) {
    if descriptor.prop(percent_property).is_none() || descriptor.prop(value_property).is_none() {
        return;
    }

    let Some(value) = super::numeric_component_value(state, descriptor, value_property) else {
        return;
    };
    let (min, max) = range_bounds(state, descriptor);
    let percent = if (max - min).abs() <= f64::EPSILON {
        0.0
    } else {
        ((value - min) / (max - min)).clamp(0.0, 1.0)
    };
    set_slider_value(state, percent_property, UiValue::Float(percent));
}

fn set_slider_value(state: &mut UiComponentState, property: &str, value: UiValue) {
    state.reference_sources.remove(property);
    if let Some(current) = state.values.get_mut(property) {
        *current = value;
    } else {
        state.values.insert(property.to_owned(), value);
    }
}

fn range_bounds(state: &UiComponentState, descriptor: &UiComponentDescriptor) -> (f64, f64) {
    let min = super::optional_numeric_setting(state, descriptor, "min", None).unwrap_or(0.0);
    let max = super::optional_numeric_setting(state, descriptor, "max", None).unwrap_or(1.0);
    if min <= max {
        (min, max)
    } else {
        (max, min)
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const PROPERTY: &str = "value_percent";
    const SAMPLE_PAIRS: usize = 17;
    const UPDATES_PER_SAMPLE: usize = 65_536;

    #[test]
    fn optimization_batch_fp_runtime472_reuses_existing_slider_value_key() {
        let mut state = UiComponentState::new();
        state
            .values
            .insert(PROPERTY.to_owned(), UiValue::Float(0.25));

        set_slider_value(&mut state, PROPERTY, UiValue::Float(0.75));

        assert_eq!(state.values.len(), 1);
        assert_eq!(state.values.get(PROPERTY), Some(&UiValue::Float(0.75)));

        let mut missing = UiComponentState::new();
        set_slider_value(&mut missing, PROPERTY, UiValue::Float(0.5));
        assert_eq!(missing.values.get(PROPERTY), Some(&UiValue::Float(0.5)));
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_fp_runtime472_borrowed_slider_value_key_benchmark() {
        for _ in 0..4 {
            black_box(measure_existing_key(false));
            black_box(measure_existing_key(true));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_existing_key(false));
                optimized_samples.push(measure_existing_key(true));
            } else {
                optimized_samples.push(measure_existing_key(true));
                legacy_samples.push(measure_existing_key(false));
            }
        }

        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "RUNTIME472_BORROWED_SLIDER_VALUE_KEY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} updates_per_sample={UPDATES_PER_SAMPLE} legacy_owned_keys_per_sample={UPDATES_PER_SAMPLE} optimized_owned_keys_per_sample=0 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=30",
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(optimized_p95 <= legacy_p95 * 70 / 100);
    }

    fn measure_existing_key(optimized: bool) -> u128 {
        let mut state = UiComponentState::new();
        state
            .values
            .insert(PROPERTY.to_owned(), UiValue::Float(0.0));
        let started = Instant::now();
        for update in 0..UPDATES_PER_SAMPLE {
            let value = UiValue::Float((update & 1_023) as f64 / 1_023.0);
            if optimized {
                set_slider_value(black_box(&mut state), PROPERTY, value);
            } else {
                legacy_set_slider_value(black_box(&mut state), PROPERTY, value);
            }
        }
        black_box(state);
        started.elapsed().as_nanos().max(1)
    }

    fn legacy_set_slider_value(state: &mut UiComponentState, property: &str, value: UiValue) {
        state.reference_sources.remove(property);
        state.values.insert(property.to_owned(), value);
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
