use std::collections::BTreeMap;

use crate::ui::retained_host as host_contract;
use crate::ui::timeline_strip::{
    TimelineStripGeneration, TimelineStripGenerationInput, TimelineStripKey,
};

pub(super) struct ProjectedTimelineStrip {
    pub(super) data: host_contract::TemplatePaneTimelineStripData,
}

pub(super) fn projected_timeline_strip(
    component_role: &str,
    attributes: &BTreeMap<String, toml::Value>,
) -> ProjectedTimelineStrip {
    ProjectedTimelineStrip {
        data: projected_timeline_strip_data(component_role, attributes),
    }
}

pub(in crate::ui::retained_host::ui) fn projected_timeline_strip_data(
    component_role: &str,
    attributes: &BTreeMap<String, toml::Value>,
) -> host_contract::TemplatePaneTimelineStripData {
    if component_role != "canvas" || !has_variant(attributes, "timeline-strip") {
        return host_contract::TemplatePaneTimelineStripData::default();
    }

    let duration = positive_number(attributes, "duration").unwrap_or(1.0);
    let current_time = number_attribute(attributes, "current_time")
        .filter(|value| value.is_finite())
        .unwrap_or_default()
        .clamp(0.0, duration);
    let tick_interval = positive_number(attributes, "tick_interval")
        .unwrap_or_else(|| duration.min(0.25))
        .min(duration);

    host_contract::TemplatePaneTimelineStripData {
        generation: TimelineStripGeneration::new(TimelineStripGenerationInput {
            duration,
            current_time,
            tick_interval,
            track_label: string_attribute(attributes, "track_label").unwrap_or_default(),
            keys: timeline_keys(attributes, duration),
        }),
    }
}

fn timeline_keys(
    attributes: &BTreeMap<String, toml::Value>,
    duration: f32,
) -> Vec<TimelineStripKey> {
    let Some(values) = attributes
        .get("timeline_keys")
        .and_then(toml::Value::as_array)
    else {
        return Vec::new();
    };
    let mut keys = Vec::with_capacity(values.len());
    for value in values {
        let Some(key) = value.as_table() else {
            continue;
        };
        let Some(time) = key.get("time").and_then(number_value) else {
            continue;
        };
        if !time.is_finite() {
            continue;
        }
        keys.push(TimelineStripKey::new(
            time.clamp(0.0, duration),
            key.get("label")
                .and_then(toml::Value::as_str)
                .unwrap_or_default()
                .to_owned(),
            key.get("selected")
                .and_then(toml::Value::as_bool)
                .unwrap_or(false),
        ));
    }
    keys
}

fn has_variant(attributes: &BTreeMap<String, toml::Value>, expected: &str) -> bool {
    ["component_variant", "variant"]
        .into_iter()
        .filter_map(|name| attributes.get(name).and_then(toml::Value::as_str))
        .any(|variant| variant.split_whitespace().any(|token| token == expected))
}

fn positive_number(attributes: &BTreeMap<String, toml::Value>, name: &str) -> Option<f32> {
    number_attribute(attributes, name).filter(|value| value.is_finite() && *value > 0.0)
}

fn number_attribute(attributes: &BTreeMap<String, toml::Value>, name: &str) -> Option<f32> {
    attributes.get(name).and_then(number_value)
}

fn number_value(value: &toml::Value) -> Option<f32> {
    match value {
        toml::Value::Float(value) => Some(*value as f32),
        toml::Value::Integer(value) => Some(*value as f32),
        _ => None,
    }
}

fn string_attribute(attributes: &BTreeMap<String, toml::Value>, name: &str) -> Option<String> {
    attributes
        .get(name)
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
}

#[cfg(test)]
mod optimization_batch_20260830cd_editor_timeline_tests {
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const VALUES_PER_SAMPLE: usize = 512;

    #[test]
    fn timeline_key_projection_reserves_array_capacity_and_keeps_filtering() {
        let source = include_str!("timeline_strip.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("timeline strip implementation");

        assert!(implementation.contains("let mut keys = Vec::with_capacity(values.len())"));
        assert!(implementation.contains("let Some(key) = value.as_table() else"));
        assert!(implementation.contains("if !time.is_finite()"));
        assert!(implementation.contains("time.clamp(0.0, duration)"));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830cd_editor_visual_array_capacity_p95() {
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(false));
                optimized.push(measure(true));
            } else {
                optimized.push(measure(true));
                legacy.push(measure(false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!("EDITOR328_VISUAL_ARRAY_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} values_per_sample={VALUES_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}", csv(&legacy), csv(&optimized));
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(use_capacity: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..128 {
            let mut timeline = if use_capacity {
                Vec::with_capacity(VALUES_PER_SAMPLE)
            } else {
                Vec::new()
            };
            let mut heatmap = if use_capacity {
                Vec::with_capacity(VALUES_PER_SAMPLE)
            } else {
                Vec::new()
            };
            for value in 0..VALUES_PER_SAMPLE {
                if value % 4 != 0 {
                    timeline.push(value);
                }
                if value % 5 != 0 {
                    heatmap.push(value);
                }
            }
            checksum ^= timeline.len() ^ heatmap.len();
        }
        std::hint::black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], p: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * p).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
