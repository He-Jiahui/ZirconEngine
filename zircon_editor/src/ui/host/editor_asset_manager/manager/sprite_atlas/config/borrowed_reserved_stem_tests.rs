use std::hint::black_box;
use std::time::Instant;

use super::{is_safe_output_stem, SpriteAtlasBuildConfig};

const CHECKS_PER_SAMPLE: usize = 8192;
const SAMPLE_PAIRS: usize = 31;
const STEM_BYTES: usize = 4096;

fn legacy_is_windows_reserved_stem(value: &str) -> bool {
    let stem = value
        .split_once('.')
        .map(|(stem, _)| stem)
        .unwrap_or(value)
        .to_ascii_uppercase();
    matches!(
        stem.as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    )
}

fn legacy_validate(config: &SpriteAtlasBuildConfig) -> Result<(), String> {
    if config.output_stem.trim().is_empty() {
        return Err("output_stem must not be empty".to_string());
    }
    if config.output_stem.trim() != config.output_stem
        || config.output_stem == "."
        || config.output_stem == ".."
        || !is_safe_output_stem(&config.output_stem)
        || config.output_stem.ends_with('.')
        || legacy_is_windows_reserved_stem(&config.output_stem)
    {
        return Err("output_stem must be a single safe file stem".to_string());
    }
    if config.initial_size.0 == 0
        || config.initial_size.1 == 0
        || config.max_size.0 == 0
        || config.max_size.1 == 0
    {
        return Err("initial_size and max_size must be non-zero".to_string());
    }
    if config.initial_size.0 > config.max_size.0 || config.initial_size.1 > config.max_size.1 {
        return Err("initial_size must fit inside max_size".to_string());
    }
    Ok(())
}

fn measure(config: &SpriteAtlasBuildConfig, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut valid = 0;
    for _ in 0..CHECKS_PER_SAMPLE {
        valid += usize::from(if optimized {
            config.validate().is_ok()
        } else {
            legacy_validate(black_box(config)).is_ok()
        });
    }
    black_box(valid);
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
fn optimization_batch_20260829bh_editor280_borrowed_reserved_stems_preserve_results() {
    for output_stem in [
        "editor-atlas",
        "editor.atlas",
        "",
        " ",
        " editor-atlas",
        "editor-atlas ",
        ".",
        "..",
        "atlas/unsafe",
        "atlas.",
        "con",
        "CoM1.texture",
        "atlas_01",
    ] {
        let config = SpriteAtlasBuildConfig {
            output_stem: output_stem.to_string(),
            ..SpriteAtlasBuildConfig::default()
        };
        assert_eq!(
            config.validate(),
            legacy_validate(&config),
            "{output_stem:?}"
        );
    }
}

#[test]
fn optimization_batch_20260829bh_editor280_reserved_stem_check_stays_borrowed() {
    let source = include_str!("../config.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;

    assert_eq!(production.matches("self.output_stem.trim()").count(), 1);
    assert!(production.contains("WINDOWS_RESERVED_STEMS"));
    assert!(production.contains("stem.eq_ignore_ascii_case(reserved)"));
    assert!(!production.contains("to_ascii_uppercase()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829bh_editor280_borrowed_reserved_atlas_stem_bench() {
    let config = SpriteAtlasBuildConfig {
        output_stem: "a".repeat(STEM_BYTES),
        ..SpriteAtlasBuildConfig::default()
    };
    let mut baseline_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline_samples.push(measure(&config, false));
            candidate_samples.push(measure(&config, true));
        } else {
            candidate_samples.push(measure(&config, true));
            baseline_samples.push(measure(&config, false));
        }
    }

    let baseline_p50_ns = percentile(&baseline_samples, 50);
    let candidate_p50_ns = percentile(&candidate_samples, 50);
    let baseline_p95_ns = percentile(&baseline_samples, 95);
    let candidate_p95_ns = percentile(&candidate_samples, 95);
    println!(
        "EDITOR280_BORROWED_RESERVED_ATLAS_STEM_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
checks_per_sample={CHECKS_PER_SAMPLE} stem_bytes={STEM_BYTES} \
baseline_uppercase_allocations={CHECKS_PER_SAMPLE} candidate_uppercase_allocations=0 \
baseline_p50_ns={baseline_p50_ns} candidate_p50_ns={candidate_p50_ns} \
baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} \
baseline_raw_ns={} candidate_raw_ns={}",
        sample_csv(&baseline_samples),
        sample_csv(&candidate_samples),
    );
    assert!(candidate_p95_ns.saturating_mul(100) <= baseline_p95_ns.saturating_mul(70));
}
