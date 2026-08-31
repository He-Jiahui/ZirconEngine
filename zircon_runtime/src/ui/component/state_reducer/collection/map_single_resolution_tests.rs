use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use super::{clear_reference_source, map_value_mut, rename_map_key};
use zircon_runtime_interface::ui::component::{
    UiComponentEventError, UiComponentState, UiValidationState, UiValue,
};

const SAMPLE_PAIRS: usize = 21;
const RENAMES_PER_SAMPLE: usize = 8_192;
const STATE_PROPERTY_COUNT: usize = 256;

type RenameFunction =
    fn(&mut UiComponentState, String, String, String) -> Result<(), UiComponentEventError>;

#[test]
fn optimization_batch_20260826el_runtime181_single_resolution_preserves_map_rename_errors() {
    let mut entries = BTreeMap::new();
    entries.insert("source".to_string(), UiValue::Float(7.0));
    entries.insert("occupied".to_string(), UiValue::Bool(true));
    let mut state = UiComponentState::new().with_value("entries", UiValue::Map(entries));

    rename_map_key(
        &mut state,
        "entries".to_string(),
        "source".to_string(),
        "renamed".to_string(),
    )
    .expect("available target should be renamed");
    let duplicate = rename_map_key(
        &mut state,
        "entries".to_string(),
        "renamed".to_string(),
        "occupied".to_string(),
    )
    .expect_err("occupied target should remain rejected");

    assert!(matches!(
        duplicate,
        UiComponentEventError::DuplicateMapKey { .. }
    ));
    let Some(UiValue::Map(entries)) = state.value("entries") else {
        panic!("entries should remain a map");
    };
    assert_eq!(entries.get("renamed"), Some(&UiValue::Float(7.0)));
    assert!(!entries.contains_key("source"));
}

#[test]
fn optimization_batch_20260826el_runtime181_map_rename_resolves_property_once() {
    let source = include_str!("../collection.rs");
    let rename_start = source.find("pub(super) fn rename_map_key").unwrap();
    let rename_end = source[rename_start..]
        .find("pub(super) fn remove_map_entry")
        .map(|offset| rename_start + offset)
        .unwrap();
    let rename_source = &source[rename_start..rename_end];

    assert!(rename_source.contains("let values = map_value_mut(state, &property);"));
    assert_eq!(rename_source.matches("map_value_mut(").count(), 1);
    assert!(rename_source.contains("values.contains_key(&to_key)"));
    assert!(rename_source.contains("values.contains_key(&from_key)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826el_runtime181_map_rename_single_resolution_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(legacy_rename_map_key));
            optimized_samples.push(measure(rename_map_key));
        } else {
            optimized_samples.push(measure(rename_map_key));
            legacy_samples.push(measure(legacy_rename_map_key));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME181_MAP_RENAME_SINGLE_RESOLUTION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
renames_per_sample={RENAMES_PER_SAMPLE} state_properties={STATE_PROPERTY_COUNT} \
legacy_property_resolutions_per_rename=3 optimized_property_resolutions_per_rename=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "single-resolution map rename P95 {optimized_p95_ns}ns must be at most 70% of repeated-resolution rename P95 {legacy_p95_ns}ns"
    );
}

fn legacy_rename_map_key(
    state: &mut UiComponentState,
    property: String,
    from_key: String,
    to_key: String,
) -> Result<(), UiComponentEventError> {
    if from_key == to_key {
        return Ok(());
    }
    if map_value_mut(state, &property).contains_key(&to_key) {
        state.validation = UiValidationState::error(format!("map key `{to_key}` already exists"));
        return Err(UiComponentEventError::DuplicateMapKey {
            property,
            key: to_key,
        });
    }
    if !map_value_mut(state, &property).contains_key(&from_key) {
        state.validation = UiValidationState::error(format!("map key `{from_key}` does not exist"));
        return Err(UiComponentEventError::MissingMapKey {
            property,
            key: from_key,
        });
    }
    let values = map_value_mut(state, &property);
    let value = values
        .remove(&from_key)
        .expect("map key was verified before rename");
    values.insert(to_key, value);
    clear_reference_source(state, &property);
    Ok(())
}

fn measure(rename: RenameFunction) -> u128 {
    let mut state = state_fixture();
    let arguments = (0..RENAMES_PER_SAMPLE)
        .map(|index| {
            if index % 2 == 0 {
                (
                    "entries".to_string(),
                    "first".to_string(),
                    "second".to_string(),
                )
            } else {
                (
                    "entries".to_string(),
                    "second".to_string(),
                    "first".to_string(),
                )
            }
        })
        .collect::<Vec<_>>();
    let started = Instant::now();
    for (property, from_key, to_key) in arguments {
        black_box(rename(black_box(&mut state), property, from_key, to_key))
            .expect("alternating fixture keys should always rename");
    }
    black_box(state.value("entries"));
    started.elapsed().as_nanos().max(1)
}

fn state_fixture() -> UiComponentState {
    let mut state = UiComponentState::new();
    for index in 0..STATE_PROPERTY_COUNT {
        state = state.with_value(format!("property-{index:03}"), UiValue::Float(index as f64));
    }
    let mut entries = BTreeMap::new();
    entries.insert("first".to_string(), UiValue::Float(1.0));
    state.with_value("entries", UiValue::Map(entries))
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
