use std::borrow::Cow;
use std::collections::BTreeMap;

use toml::Value;

pub(super) fn origin_axis<'a>(
    attributes: &'a BTreeMap<String, Value>,
    key: &str,
    default: &'a str,
) -> Cow<'a, str> {
    attributes
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(Cow::Borrowed)
        .unwrap_or(Cow::Borrowed(default))
}

pub(super) fn default_anchor_origin_vertical(component_role: &str) -> &'static str {
    match component_role {
        "menu" | "context-menu" | "context-action-menu" | "dropdown-popup" => "bottom",
        _ => "top",
    }
}

pub(super) fn default_anchor_origin_horizontal(_component_role: &str) -> &'static str {
    "left"
}

pub(super) fn default_transform_origin_vertical(_component_role: &str) -> &'static str {
    "top"
}

pub(super) fn default_transform_origin_horizontal(_component_role: &str) -> &'static str {
    "left"
}

pub(super) fn origin_offset(length: f32, axis: &str) -> f32 {
    match axis {
        "center" => length * 0.5,
        "bottom" | "right" | "end" => length,
        value => value.parse::<f32>().unwrap_or(0.0),
    }
}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    #[test]
    fn optimization_batch_gx_editor579_origin_axis_borrows_defaults() {
        let attributes = BTreeMap::new();
        let origin = origin_axis(&attributes, "missing", "top");

        assert_eq!(origin.as_ref(), "top");
        assert!(matches!(origin, Cow::Borrowed(_)));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_gx_editor579_origin_axis_borrowed_defaults_p95() {
        const SAMPLE_PAIRS: usize = 21;
        const ITERATIONS: usize = 1_000_000;
        let attributes = BTreeMap::new();
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure_legacy(&attributes, ITERATIONS));
                optimized.push(measure_optimized(&attributes, ITERATIONS));
            } else {
                optimized.push(measure_optimized(&attributes, ITERATIONS));
                legacy.push(measure_legacy(&attributes, ITERATIONS));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "EDITOR579_ORIGIN_AXIS_BORROWED_DEFAULTS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
iterations={ITERATIONS} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy),
            csv(&optimized)
        );
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(90),
            "borrowed popup defaults must improve P95 by at least 10%"
        );
    }

    fn measure_legacy(attributes: &BTreeMap<String, Value>, iterations: usize) -> u128 {
        let started = Instant::now();
        let mut length = 0;
        for _ in 0..iterations {
            length += legacy_origin_axis(attributes, "anchor_origin_vertical", "top").len();
            length += legacy_origin_axis(attributes, "anchor_origin_horizontal", "left").len();
            length += legacy_origin_axis(attributes, "transform_origin_vertical", "top").len();
            length += legacy_origin_axis(attributes, "transform_origin_horizontal", "left").len();
        }
        black_box(length);
        started.elapsed().as_nanos().max(1)
    }

    fn measure_optimized(attributes: &BTreeMap<String, Value>, iterations: usize) -> u128 {
        let started = Instant::now();
        let mut length = 0;
        for _ in 0..iterations {
            length += origin_axis(attributes, "anchor_origin_vertical", "top").len();
            length += origin_axis(attributes, "anchor_origin_horizontal", "left").len();
            length += origin_axis(attributes, "transform_origin_vertical", "top").len();
            length += origin_axis(attributes, "transform_origin_horizontal", "left").len();
        }
        black_box(length);
        started.elapsed().as_nanos().max(1)
    }

    fn legacy_origin_axis(
        attributes: &BTreeMap<String, Value>,
        key: &str,
        default: &str,
    ) -> String {
        attributes
            .get(key)
            .and_then(|value| value.as_str().map(str::to_owned))
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| default.to_string())
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
