use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::design_tokens::EditorTypographyTokens;

use super::{project_font_weights, project_typography_tokens, valid_font_weight_or};

const SAMPLE_PAIRS: usize = 31;
const PROJECTIONS_PER_SAMPLE: usize = 100_000;

#[test]
fn optimization_batch_20260828iu_editor239_valid_weights_preserve_projection() {
    let mut tokens = EditorTypographyTokens::workbench_default();
    tokens.body_weight = 420;
    tokens.strong_weight = 650;
    tokens.code_weight = 430;

    let preferences = project_typography_tokens(&tokens);

    assert_eq!(preferences.ui_weight, 420);
    assert_eq!(preferences.strong_weight, 650);
    assert_eq!(preferences.code_weight, 430);
    assert_eq!(project_font_weights(&tokens), (420, 650, 430));

    let source = include_str!("../typography.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    let helper = implementation
        .split("fn project_font_weights")
        .nth(1)
        .and_then(|body| body.split("fn valid_font_weight_or").next())
        .expect("font-weight projection");
    let fast_return = helper.find("return (").expect("valid fast return");
    let defaults = helper
        .find("EditorTypographyTokens::workbench_default()")
        .expect("invalid fallback defaults");
    assert!(fast_return < defaults);
}

#[test]
fn optimization_batch_20260828iu_editor239_invalid_weights_keep_workbench_defaults() {
    let mut tokens = EditorTypographyTokens::workbench_default();
    tokens.body_weight = 0;
    tokens.strong_weight = 1_001;
    tokens.code_weight = u16::MAX;
    let defaults = EditorTypographyTokens::workbench_default();

    assert_eq!(
        project_font_weights(&tokens),
        (
            defaults.body_weight,
            defaults.strong_weight,
            defaults.code_weight,
        )
    );
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260828iu_editor239_lazy_typography_default_weights_bench() {
    let tokens = EditorTypographyTokens::workbench_default();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&tokens, false));
            optimized_samples.push(measure(&tokens, true));
        } else {
            optimized_samples.push(measure(&tokens, true));
            legacy_samples.push(measure(&tokens, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR239_LAZY_TYPOGRAPHY_DEFAULT_WEIGHTS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
projections_per_sample={PROJECTIONS_PER_SAMPLE} \
legacy_default_string_allocations_per_projection=3 optimized_default_string_allocations_per_projection=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_project_font_weights(tokens: &EditorTypographyTokens) -> (u16, u16, u16) {
    let defaults = EditorTypographyTokens::workbench_default();
    (
        valid_font_weight_or(tokens.body_weight, defaults.body_weight),
        valid_font_weight_or(tokens.strong_weight, defaults.strong_weight),
        valid_font_weight_or(tokens.code_weight, defaults.code_weight),
    )
}

fn measure(tokens: &EditorTypographyTokens, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for iteration in 0..PROJECTIONS_PER_SAMPLE {
        let weights = if optimized {
            project_font_weights(black_box(tokens))
        } else {
            legacy_project_font_weights(black_box(tokens))
        };
        checksum ^= black_box(
            usize::from(weights.0)
                .wrapping_add(usize::from(weights.1))
                .wrapping_add(usize::from(weights.2))
                .wrapping_add(iteration),
        );
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

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
