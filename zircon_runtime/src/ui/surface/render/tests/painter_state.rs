use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 17;
const RESOLUTIONS_PER_SAMPLE: usize = 262_144;

#[test]
fn static_focus_preview_yields_to_runtime_focus_visibility() {
    let mut metadata = UiTemplateNodeMetadata::default();
    metadata
        .attributes
        .insert("focused".to_string(), Value::Boolean(true));
    let state_flags = UiStateFlags {
        enabled: true,
        ..UiStateFlags::default()
    };

    let static_preview =
        UiRenderPainterStateSource::new(Some(&metadata), &state_flags, None).painter_state();
    assert!(static_preview.focused);
    assert!(static_preview.focus_visible);

    let mut pointer_focus = UiComponentState::default();
    pointer_focus.flags.focused = true;
    let pointer =
        UiRenderPainterStateSource::new(Some(&metadata), &state_flags, Some(&pointer_focus))
            .painter_state();
    assert!(pointer.focused);
    assert!(!pointer.focus_visible);

    pointer_focus.flags.focus_visible = true;
    let keyboard =
        UiRenderPainterStateSource::new(Some(&metadata), &state_flags, Some(&pointer_focus))
            .painter_state();
    assert!(keyboard.focus_visible);
}

#[test]
fn optimization_batch_fr_runtime474_reads_static_focus_once() {
    let source = include_str!("../painter_state.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("painter-state production source");

    assert_eq!(
        production
            .matches("metadata_focused(source.metadata)")
            .count(),
        1
    );
    assert_eq!(
        production
            .matches("bool_attribute(metadata, \"focused\")")
            .count(),
        1
    );
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_fr_runtime474_single_static_focus_lookup_benchmark() {
    let mut metadata = UiTemplateNodeMetadata::default();
    for index in 0..16 {
        metadata.attributes.insert(
            format!("representative_attribute_{index:02}"),
            Value::Boolean(index == 7),
        );
    }
    metadata
        .attributes
        .insert("focused".to_owned(), Value::Boolean(true));

    for _ in 0..4 {
        black_box(measure_focus_resolution(&metadata, false));
        black_box(measure_focus_resolution(&metadata, true));
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_focus_resolution(&metadata, false));
            optimized_samples.push(measure_focus_resolution(&metadata, true));
        } else {
            optimized_samples.push(measure_focus_resolution(&metadata, true));
            legacy_samples.push(measure_focus_resolution(&metadata, false));
        }
    }

    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "RUNTIME474_SINGLE_STATIC_FOCUS_LOOKUP_BENCH_V1 sample_pairs={SAMPLE_PAIRS} resolutions_per_sample={RESOLUTIONS_PER_SAMPLE} metadata_attributes={} legacy_focused_lookups_per_resolution=2 optimized_focused_lookups_per_resolution=1 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=35",
        metadata.attributes.len(),
        csv(&legacy_samples),
        csv(&optimized_samples),
    );
    assert!(optimized_p95 <= legacy_p95 * 65 / 100);
}

fn measure_focus_resolution(metadata: &UiTemplateNodeMetadata, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = false;
    for _ in 0..RESOLUTIONS_PER_SAMPLE {
        if optimized {
            let focused = metadata_focused(black_box(Some(metadata)));
            checksum ^= focused;
        } else {
            let focused = bool_attribute(black_box(Some(metadata)), "focused").unwrap_or(false);
            let fallback = bool_attribute(black_box(Some(metadata)), "focused").unwrap_or(false);
            checksum ^= focused | fallback;
        }
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

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
