use std::hint::black_box;
use std::time::{Duration, Instant};

use super::{camel_to_snake_segment, component_showcase_action_id_for_binding_id};

const PERFORMANCE_MARKER: &str = "EDITOR889_COMPONENT_SHOWCASE_ACTION_ID_SINGLE_BUFFER_BENCH_V1";
const SAMPLE_PAIRS: usize = 101;
const RENDERS_PER_SAMPLE: usize = 4_096;

#[test]
fn editor889_component_showcase_action_id_single_buffer_preserves_exact_text() {
    for binding_id in [
        "UiComponentShowcase/SetValue",
        "UiComponentShowcase/--Set--Value--",
        "Other/SetValue.Action:Now",
        "Other//SetValue",
        "Other/---/Value",
        "///",
        "\u{00c5}ngstromValue",
    ] {
        assert_eq!(
            component_showcase_action_id_for_binding_id(binding_id),
            legacy_component_showcase_action_id_for_binding_id(binding_id),
            "binding_id={binding_id:?}"
        );
    }

    assert_eq!(
        component_showcase_action_id_for_binding_id("Other/---/Value"),
        "other..value"
    );
    assert_eq!(camel_to_snake_segment("--Set--Value--"), "set_value");
}

#[test]
#[ignore = "release-only component showcase action-id performance gate"]
fn editor889_component_showcase_action_id_single_buffer_release_performance() {
    let binding_id = (0..64)
        .map(|index| format!("MaterialPreviewRuntimeBindingAction{index:08}SelectedValueChanged"))
        .collect::<Vec<_>>()
        .join("/");
    let expected = legacy_component_showcase_action_id_for_binding_id(&binding_id);
    assert_eq!(
        component_showcase_action_id_for_binding_id(&binding_id),
        expected
    );

    for _ in 0..8 {
        black_box(render_batch(
            &binding_id,
            legacy_component_showcase_action_id_for_binding_id,
        ));
        black_box(render_batch(
            &binding_id,
            component_showcase_action_id_for_binding_id,
        ));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample in 0..SAMPLE_PAIRS {
        if sample % 2 == 0 {
            legacy_samples.push(measure(|| {
                render_batch(
                    &binding_id,
                    legacy_component_showcase_action_id_for_binding_id,
                )
            }));
            optimized_samples.push(measure(|| {
                render_batch(&binding_id, component_showcase_action_id_for_binding_id)
            }));
        } else {
            optimized_samples.push(measure(|| {
                render_batch(&binding_id, component_showcase_action_id_for_binding_id)
            }));
            legacy_samples.push(measure(|| {
                render_batch(
                    &binding_id,
                    legacy_component_showcase_action_id_for_binding_id,
                )
            }));
        }
    }

    let legacy_p50_ns = percentile_ns(&mut legacy_samples, 50);
    let legacy_p95_ns = percentile_ns(&mut legacy_samples, 95);
    let legacy_p99_ns = percentile_ns(&mut legacy_samples, 99);
    let optimized_p50_ns = percentile_ns(&mut optimized_samples, 50);
    let optimized_p95_ns = percentile_ns(&mut optimized_samples, 95);
    let optimized_p99_ns = percentile_ns(&mut optimized_samples, 99);
    println!(
        "{PERFORMANCE_MARKER} legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_p99_ns={legacy_p99_ns} optimized_p99_ns={optimized_p99_ns} sample_pairs={SAMPLE_PAIRS} renders_per_sample={RENDERS_PER_SAMPLE} segments_per_render=64 legacy_child_strings_per_sample=262144 optimized_child_strings_per_sample=0 legacy_vector_slots_per_sample=262144 optimized_vector_slots_per_sample=0"
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(110),
        "single-buffer P95 {optimized_p95_ns}ns must be at most 110% of collect/join P95 {legacy_p95_ns}ns"
    );
}

fn legacy_component_showcase_action_id_for_binding_id(binding_id: &str) -> String {
    let Some(suffix) = binding_id.strip_prefix("UiComponentShowcase/") else {
        return binding_id
            .split(['/', '.', ':'])
            .filter(|segment| !segment.is_empty())
            .map(legacy_camel_to_snake_segment)
            .collect::<Vec<_>>()
            .join(".");
    };
    format!(
        "ui_component_showcase.{}",
        legacy_camel_to_snake_segment(suffix)
    )
}

fn legacy_camel_to_snake_segment(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut previous_was_separator = true;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() && !previous_was_separator && !output.ends_with('_') {
                output.push('_');
            }
            output.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !output.is_empty() && !output.ends_with('_') {
            output.push('_');
            previous_was_separator = true;
        }
    }
    if output.ends_with('_') {
        output.pop();
    }
    output
}

fn render_batch(binding_id: &str, render: fn(&str) -> String) -> usize {
    (0..RENDERS_PER_SAMPLE)
        .map(|_| black_box(render(black_box(binding_id))).len())
        .sum()
}

fn measure<T>(run: impl FnOnce() -> T) -> Duration {
    let started = Instant::now();
    black_box(run());
    started.elapsed()
}

fn percentile_ns(samples: &mut [Duration], percentile: usize) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * percentile).div_ceil(100);
    samples[rank.saturating_sub(1)].as_nanos()
}
