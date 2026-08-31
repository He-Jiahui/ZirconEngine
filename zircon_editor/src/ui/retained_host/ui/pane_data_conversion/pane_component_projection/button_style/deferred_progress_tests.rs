use std::borrow::Cow;
use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use super::super::{
    attribute_is_true, button_style_values_with_aliases, is_progress_component_role,
    progress_fill_color_source, progress_track_color_source,
};

const ATTRIBUTE_COUNT: usize = 1024;
const CHECKS_PER_SAMPLE: usize = 8192;
const SAMPLE_PAIRS: usize = 31;

fn legacy_button_style_values_with_aliases<'a>(
    attributes: &'a BTreeMap<String, toml::Value>,
    component_role: &str,
) -> Cow<'a, BTreeMap<String, toml::Value>> {
    let progress_aliases = is_progress_component_role(component_role);
    let progress_state_override = progress_aliases && attribute_is_true(attributes, "disabled");
    let progress_track_source = progress_track_color_source(attributes);
    let progress_fill_source = progress_fill_color_source(attributes);
    let needs_alias = [
        ("focus_border_color", "border_color"),
        ("thumb_outline_color", "border_color"),
        ("disabled_opacity", "opacity"),
    ]
    .into_iter()
    .any(|(source, target)| attributes.contains_key(source) && !attributes.contains_key(target))
        || progress_aliases
            && [
                (progress_track_source, "background_color"),
                (progress_fill_source, "foreground_color"),
            ]
            .into_iter()
            .any(|(source, target)| {
                source.is_some_and(|source| {
                    attributes.contains_key(source)
                        && (progress_state_override || !attributes.contains_key(target))
                })
            });
    if needs_alias {
        Cow::Owned(attributes.clone())
    } else {
        Cow::Borrowed(attributes)
    }
}

fn fixture_attributes() -> BTreeMap<String, toml::Value> {
    (0..ATTRIBUTE_COUNT)
        .map(|index| {
            (
                format!("custom_attribute_{index:04}"),
                toml::Value::Integer(index as i64),
            )
        })
        .collect()
}

fn measure(attributes: &BTreeMap<String, toml::Value>, optimized: bool) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        let values = if optimized {
            button_style_values_with_aliases(black_box(attributes), "button")
        } else {
            legacy_button_style_values_with_aliases(black_box(attributes), "button")
        };
        black_box(matches!(values, Cow::Borrowed(_)));
    }
    started.elapsed().as_nanos().max(1)
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

#[test]
fn optimization_batch_20260829ba_editor273_non_progress_styles_remain_borrowed() {
    let attributes = BTreeMap::from([
        (
            "track_color".to_owned(),
            toml::Value::String("#123456".to_owned()),
        ),
        (
            "fill_color".to_owned(),
            toml::Value::String("#abcdef".to_owned()),
        ),
    ]);

    let values = button_style_values_with_aliases(&attributes, "button");
    assert!(matches!(values, Cow::Borrowed(_)));
    assert_eq!(attributes.len(), 2);
}

#[test]
fn optimization_batch_20260829ba_editor273_progress_probes_are_role_gated() {
    let source = include_str!("../button_style.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;

    assert!(production.contains("if progress_aliases {"));
    assert!(production.contains("progress_track_color_source(attributes)"));
    assert!(production.contains("progress_fill_color_source(attributes)"));
    assert!(production.contains("} else {\n        (None, None)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829ba_editor273_deferred_progress_alias_probes_bench() {
    let attributes = fixture_attributes();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&attributes, false));
            optimized_samples.push(measure(&attributes, true));
        } else {
            optimized_samples.push(measure(&attributes, true));
            legacy_samples.push(measure(&attributes, false));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR273_DEFERRED_PROGRESS_ALIAS_PROBES_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
checks_per_sample={CHECKS_PER_SAMPLE} attributes={ATTRIBUTE_COUNT} \
legacy_non_progress_probes_per_check=8 optimized_non_progress_probes_per_check=3 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}
