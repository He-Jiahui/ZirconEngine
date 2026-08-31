use toml::Value;
use zircon_runtime_interface::ui::{surface::UiTextAlign, tree::UiTemplateNodeMetadata};

pub(super) fn value_to_display_text(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        Value::Integer(value) => value.to_string(),
        Value::Float(value) => value.to_string(),
        Value::Boolean(value) => value.to_string(),
        _ => value.to_string(),
    }
}

pub(super) fn string_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<String> {
    metadata
        .attributes
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
}

pub(super) fn string_array_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Vec<String> {
    metadata
        .attributes
        .get(key)
        .and_then(Value::as_array)
        .map(|values| {
            let mut strings = Vec::with_capacity(values.len());
            for value in values {
                if let Some(value) = value.as_str() {
                    strings.push(value.to_string());
                }
            }
            strings
        })
        .unwrap_or_default()
}

pub(super) fn number_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<f32> {
    metadata.attributes.get(key).and_then(|value| match value {
        Value::Float(value) => Some(*value as f32),
        Value::Integer(value) => Some(*value as f32),
        _ => None,
    })
}

pub(super) fn bool_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<bool> {
    metadata.attributes.get(key).and_then(Value::as_bool)
}

pub(super) fn integer_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<i32> {
    metadata
        .attributes
        .get(key)
        .and_then(Value::as_integer)
        .map(|value| value as i32)
}

pub(super) fn text_align_name(align: UiTextAlign) -> &'static str {
    match align {
        UiTextAlign::Left => "left",
        UiTextAlign::Center => "center",
        UiTextAlign::Right => "right",
        UiTextAlign::Start => "start",
        UiTextAlign::End => "end",
        UiTextAlign::Justify => "justify",
    }
}

#[cfg(test)]
mod optimization_batch_20260830cg_editor_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 17;
    const VALUES_PER_SAMPLE: usize = 512;

    #[test]
    fn string_array_projection_preserves_string_order_and_empty_values() {
        let mut metadata = UiTemplateNodeMetadata::default();
        metadata.attributes.insert(
            "options".to_string(),
            Value::Array(vec![
                Value::String("first".to_string()),
                Value::Integer(7),
                Value::String(String::new()),
                Value::Boolean(true),
                Value::String("last".to_string()),
            ]),
        );

        assert_eq!(
            string_array_attribute(&metadata, "options"),
            vec!["first".to_string(), String::new(), "last".to_string()]
        );
    }

    #[test]
    fn string_array_projection_reserves_array_capacity() {
        let source = include_str!("metadata.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("view projection metadata implementation");

        assert!(implementation.contains("Vec::with_capacity(values.len())"));
        assert!(implementation.contains("for value in values"));
        assert!(implementation.contains("if let Some(value) = value.as_str()"));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830cg_editor_string_array_capacity_p95() {
        let values = (0..VALUES_PER_SAMPLE)
            .map(|index| (index % 4 != 0).then_some(index))
            .collect::<Vec<_>>();
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(&values, false));
                optimized.push(measure(&values, true));
            } else {
                optimized.push(measure(&values, true));
                legacy.push(measure(&values, false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!("EDITOR331_STRING_ARRAY_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} values_per_sample={VALUES_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}", csv(&legacy), csv(&optimized));
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(values: &[Option<usize>], use_capacity: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..2_048 {
            let projected = if use_capacity {
                let mut projected = Vec::with_capacity(values.len());
                for value in black_box(values) {
                    if let Some(value) = value {
                        projected.push(*value);
                    }
                }
                projected
            } else {
                black_box(values)
                    .iter()
                    .filter_map(|value| *value)
                    .collect()
            };
            checksum ^= projected.len();
            black_box(projected);
        }
        black_box(checksum);
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
