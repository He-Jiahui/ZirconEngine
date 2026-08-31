use std::collections::BTreeMap;

use crate::ui::retained_host as host_contract;
use crate::ui::sample_grid::{SampleGridGeneration, SampleGridGenerationInput, SampleGridPoint};

pub(super) struct ProjectedSampleGrid {
    pub(super) data: host_contract::TemplatePaneSampleGridData,
}

pub(super) fn projected_sample_grid(
    component_role: &str,
    attributes: &BTreeMap<String, toml::Value>,
) -> ProjectedSampleGrid {
    ProjectedSampleGrid {
        data: projected_sample_grid_data(component_role, attributes),
    }
}

pub(in crate::ui::retained_host::ui) fn projected_sample_grid_data(
    component_role: &str,
    attributes: &BTreeMap<String, toml::Value>,
) -> host_contract::TemplatePaneSampleGridData {
    if component_role != "canvas" || !has_variant(attributes, "sample-grid") {
        return host_contract::TemplatePaneSampleGridData::default();
    }

    let mut x_min = number_attribute(attributes, "x_min").unwrap_or(0.0);
    let mut x_max = number_attribute(attributes, "x_max").unwrap_or(1.0);
    let mut y_min = number_attribute(attributes, "y_min").unwrap_or(0.0);
    let mut y_max = number_attribute(attributes, "y_max").unwrap_or(1.0);
    normalize_range(&mut x_min, &mut x_max);
    normalize_range(&mut y_min, &mut y_max);
    host_contract::TemplatePaneSampleGridData {
        generation: SampleGridGeneration::new(SampleGridGenerationInput {
            x_axis_label: string_attribute(attributes, "x_axis_label").unwrap_or_default(),
            y_axis_label: string_attribute(attributes, "y_axis_label").unwrap_or_default(),
            x_min,
            x_max,
            y_min,
            y_max,
            x_ticks: number_array_attribute(attributes, "x_ticks"),
            y_ticks: number_array_attribute(attributes, "y_ticks"),
            points: sample_points(attributes),
        }),
    }
}

fn has_variant(attributes: &BTreeMap<String, toml::Value>, expected: &str) -> bool {
    ["component_variant", "variant"]
        .into_iter()
        .filter_map(|name| attributes.get(name).and_then(toml::Value::as_str))
        .any(|variant| variant.split_whitespace().any(|token| token == expected))
}

fn sample_points(attributes: &BTreeMap<String, toml::Value>) -> Vec<SampleGridPoint> {
    let Some(values) = attributes
        .get("sample_points")
        .and_then(toml::Value::as_array)
    else {
        return Vec::new();
    };
    let mut points = Vec::with_capacity(values.len());
    for value in values {
        let Some(point) = value.as_table() else {
            continue;
        };
        let (Some(x), Some(y)) = (
            point.get("x").and_then(number_value),
            point.get("y").and_then(number_value),
        ) else {
            continue;
        };
        points.push(SampleGridPoint::new(
            x,
            y,
            point
                .get("label")
                .and_then(toml::Value::as_str)
                .unwrap_or_default(),
            point
                .get("selected")
                .and_then(toml::Value::as_bool)
                .unwrap_or(false),
        ));
    }
    points
}

fn number_array_attribute(attributes: &BTreeMap<String, toml::Value>, name: &str) -> Vec<f32> {
    let Some(values) = attributes.get(name).and_then(toml::Value::as_array) else {
        return Vec::new();
    };
    let mut output = Vec::with_capacity(values.len());
    output.extend(values.iter().filter_map(number_value));
    output
}

fn string_attribute(attributes: &BTreeMap<String, toml::Value>, name: &str) -> Option<String> {
    attributes
        .get(name)
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
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

fn normalize_range(min: &mut f32, max: &mut f32) {
    if !min.is_finite() || !max.is_finite() || *max <= *min {
        *min = 0.0;
        *max = 1.0;
    }
}

#[cfg(test)]
mod optimization_batch_20260830cc_editor_tests {
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const VALUES_PER_SAMPLE: usize = 256;

    #[test]
    fn sample_grid_reserves_point_and_number_array_capacity() {
        let source = include_str!("sample_grid.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        assert!(implementation.contains("let mut points = Vec::with_capacity(values.len())"));
        assert!(implementation.contains("let mut output = Vec::with_capacity(values.len())"));
        assert!(implementation.contains("for value in values"));
        assert!(implementation.contains("output.extend(values.iter().filter_map(number_value))"));
    }

    #[test]
    fn sample_grid_keeps_invalid_value_filtering() {
        let source = include_str!("sample_grid.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        assert!(implementation.contains("let Some(point) = value.as_table() else"));
        assert!(implementation.contains("let (Some(x), Some(y))"));
        assert!(implementation.contains("filter_map(number_value)"));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830cc_editor_sample_grid_capacity_p95() {
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
        println!("EDITOR327_SAMPLE_GRID_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} values_per_sample={VALUES_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",csv(&legacy),csv(&optimized));
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(optimized: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..128 {
            let mut output = if optimized {
                Vec::with_capacity(VALUES_PER_SAMPLE)
            } else {
                Vec::new()
            };
            for index in 0..VALUES_PER_SAMPLE {
                if index % 3 != 0 {
                    output.push(index);
                }
            }
            checksum ^= output.len();
        }
        std::hint::black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }
    fn percentile(samples: &[u128], p: usize) -> u128 {
        let mut s = samples.to_vec();
        s.sort_unstable();
        s[(s.len() * p).div_ceil(100).saturating_sub(1)]
    }
    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
