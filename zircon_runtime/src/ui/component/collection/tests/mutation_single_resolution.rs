use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::component::{UiComponentEventError, UiComponentState, UiValue};

use super::{
    add_map_entry, array_value_mut, move_array_element, remove_array_element, remove_map_entry,
    set_array_element, set_map_entry,
};

const MUTATIONS_PER_SAMPLE: usize = 4_096;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn runtime770_collection_mutations_preserve_success_and_error_contracts() {
    let mut state = UiComponentState::new().with_value(
        "items",
        UiValue::Array(vec![
            UiValue::String("first".to_string()),
            UiValue::String("second".to_string()),
        ]),
    );
    set_array_element(
        &mut state,
        "items".to_string(),
        1,
        UiValue::String("updated".to_string()),
    )
    .expect("existing array entry should update");
    move_array_element(&mut state, "items".to_string(), 1, 0)
        .expect("existing array entry should move");
    remove_array_element(&mut state, "items".to_string(), 1)
        .expect("existing array entry should remove");
    assert_eq!(
        state.value("items"),
        Some(&UiValue::Array(vec![UiValue::String(
            "updated".to_string()
        )]))
    );
    let missing_array = set_array_element(
        &mut state,
        "items".to_string(),
        1,
        UiValue::String("missing".to_string()),
    )
    .expect_err("missing array entry should remain rejected");
    assert!(matches!(
        missing_array,
        UiComponentEventError::ArrayIndexOutOfBounds { index: 1, .. }
    ));

    let mut entries = BTreeMap::new();
    entries.insert("existing".to_string(), UiValue::Int(1));
    state
        .values
        .insert("entries".to_string(), UiValue::Map(entries));
    add_map_entry(
        &mut state,
        "entries".to_string(),
        "added".to_string(),
        UiValue::Int(2),
    )
    .expect("new map key should insert");
    set_map_entry(
        &mut state,
        "entries".to_string(),
        "existing".to_string(),
        UiValue::Int(3),
    )
    .expect("existing map key should update");
    let duplicate = add_map_entry(
        &mut state,
        "entries".to_string(),
        "added".to_string(),
        UiValue::Int(4),
    )
    .expect_err("duplicate map key should remain rejected");
    assert!(matches!(
        duplicate,
        UiComponentEventError::DuplicateMapKey { .. }
    ));
    let missing_map = set_map_entry(
        &mut state,
        "entries".to_string(),
        "missing".to_string(),
        UiValue::Int(5),
    )
    .expect_err("missing map key should remain rejected");
    assert!(matches!(
        missing_map,
        UiComponentEventError::MissingMapKey { .. }
    ));
    remove_map_entry(&mut state, "entries".to_string(), "added".to_string())
        .expect("existing map key should remove");
    let missing_remove = remove_map_entry(&mut state, "entries".to_string(), "added".to_string())
        .expect_err("removed map key should remain rejected");
    assert!(matches!(
        missing_remove,
        UiComponentEventError::MissingMapKey { .. }
    ));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime770_collection_single_resolution_release_benchmark() {
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_set_array(true));
            optimized_ns.push(measure_set_array(false));
        } else {
            optimized_ns.push(measure_set_array(false));
            legacy_ns.push(measure_set_array(true));
        }
    }

    let legacy_property_resolutions = MUTATIONS_PER_SAMPLE * 2;
    let optimized_property_resolutions = MUTATIONS_PER_SAMPLE;
    assert!(legacy_property_resolutions > optimized_property_resolutions);
    assert_eq!(optimized_property_resolutions, MUTATIONS_PER_SAMPLE);

    println!(
        "RUNTIME770_COLLECTION_SINGLE_RESOLUTION_BENCH_V1 mutations_per_sample={MUTATIONS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_property_resolutions={legacy_property_resolutions} optimized_property_resolutions={optimized_property_resolutions} legacy_p95_ns={} optimized_p95_ns={}",
        percentile(&legacy_ns, 95),
        percentile(&optimized_ns, 95),
    );
}

fn measure_set_array(legacy: bool) -> u128 {
    let mut state =
        UiComponentState::new().with_value("items", UiValue::Array(vec![UiValue::Int(0)]));
    let started = Instant::now();
    for index in 0..MUTATIONS_PER_SAMPLE {
        let value = UiValue::Int(index as i64);
        if legacy {
            legacy_set_array_element(black_box(&mut state), "items".to_string(), 0, value)
                .expect("fixture index should remain valid");
        } else {
            set_array_element(black_box(&mut state), "items".to_string(), 0, value)
                .expect("fixture index should remain valid");
        }
    }
    black_box(state);
    started.elapsed().as_nanos().max(1)
}

fn legacy_set_array_element(
    state: &mut UiComponentState,
    property: String,
    index: usize,
    value: UiValue,
) -> Result<(), UiComponentEventError> {
    if index >= array_value_mut(state, &property).len() {
        return Err(UiComponentEventError::ArrayIndexOutOfBounds { property, index });
    }
    array_value_mut(state, &property)[index] = value;
    super::clear_reference_source(state, &property);
    Ok(())
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
