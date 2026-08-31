use super::super::pane_value_conversion::normalized_value_percent;

pub(super) fn projected_value_percent(
    component_role: &str,
    value_number: f64,
    value_percent: Option<f64>,
    min: Option<f64>,
    max: Option<f64>,
) -> f32 {
    let role = value_component_role(component_role);
    if role == ValueComponentRole::Slider {
        if let (Some(min), Some(max)) = (min, max) {
            if max > min {
                return ((value_number - min) / (max - min)).clamp(0.0, 1.0) as f32;
            }
        }
    }
    if let Some(value_percent) = value_percent {
        return normalize_percent_literal(value_percent);
    }
    match (min, max) {
        (Some(min), Some(max)) if max > min => {
            ((value_number - min) / (max - min)).clamp(0.0, 1.0) as f32
        }
        _ if role == ValueComponentRole::Progress && value_number > 1.0 => {
            normalize_percent_literal(value_number)
        }
        _ => normalized_value_percent(value_number, min, max),
    }
}

fn normalize_percent_literal(value: f64) -> f32 {
    if value > 1.0 {
        (value / 100.0).clamp(0.0, 1.0) as f32
    } else {
        value.clamp(0.0, 1.0) as f32
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ValueComponentRole {
    Slider,
    Progress,
    Other,
}

fn value_component_role(component_role: &str) -> ValueComponentRole {
    match component_role {
        "range-field" | "slider" | "range-slider" => ValueComponentRole::Slider,
        "progress" | "progress-bar" | "linear-progress" | "circular-progress" | "spinner" => {
            ValueComponentRole::Progress
        }
        _ => ValueComponentRole::Other,
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 31;
    const ITERATIONS: usize = 2_000_000;

    #[test]
    fn interactive_slider_values_override_static_preview_percents() {
        for component_role in ["range-field", "range-slider"] {
            assert_eq!(
                projected_value_percent(component_role, 30.0, Some(0.8), Some(0.0), Some(100.0)),
                0.3,
            );
        }
    }

    #[test]
    fn optimization_batch_gv_editor577_value_role_dispatch_preserves_families() {
        for role in ["range-field", "slider", "range-slider"] {
            assert_eq!(value_component_role(role), ValueComponentRole::Slider);
        }
        for role in [
            "progress",
            "progress-bar",
            "linear-progress",
            "circular-progress",
            "spinner",
        ] {
            assert_eq!(value_component_role(role), ValueComponentRole::Progress);
            assert_eq!(projected_value_percent(role, 35.0, None, None, None), 0.35);
        }
        assert_eq!(value_component_role("unknown"), ValueComponentRole::Other);
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_gv_editor577_progress_role_single_dispatch_p95() {
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure(false));
                optimized_samples.push(measure(true));
            } else {
                optimized_samples.push(measure(true));
                legacy_samples.push(measure(false));
            }
        }

        let legacy_p95_ns = p95(&mut legacy_samples);
        let optimized_p95_ns = p95(&mut optimized_samples);
        println!(
            "EDITOR577_PROGRESS_ROLE_SINGLE_DISPATCH_BENCH_V1 sample_pairs={SAMPLE_PAIRS} iterations={ITERATIONS} role=circular-progress legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(90),
            "expected single role dispatch to lower p95 by at least 10%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    fn measure(optimized: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0_f32;
        for _ in 0..ITERATIONS {
            let role = black_box("circular-progress");
            let percent = if optimized {
                projected_value_percent(role, 35.0, None, None, None)
            } else {
                legacy_projected_value_percent(role, 35.0, None, None, None)
            };
            checksum += percent;
        }
        black_box(checksum);
        started.elapsed().as_nanos()
    }

    fn legacy_projected_value_percent(
        component_role: &str,
        value_number: f64,
        value_percent: Option<f64>,
        min: Option<f64>,
        max: Option<f64>,
    ) -> f32 {
        if matches!(component_role, "range-field" | "slider" | "range-slider") {
            if let (Some(min), Some(max)) = (min, max) {
                if max > min {
                    return ((value_number - min) / (max - min)).clamp(0.0, 1.0) as f32;
                }
            }
        }
        if let Some(value_percent) = value_percent {
            return normalize_percent_literal(value_percent);
        }
        match (min, max) {
            (Some(min), Some(max)) if max > min => {
                ((value_number - min) / (max - min)).clamp(0.0, 1.0) as f32
            }
            _ if matches!(
                component_role,
                "progress" | "progress-bar" | "linear-progress" | "circular-progress" | "spinner"
            ) && value_number > 1.0 =>
            {
                normalize_percent_literal(value_number)
            }
            _ => normalized_value_percent(value_number, min, max),
        }
    }

    fn p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[samples.len() * 95 / 100]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
