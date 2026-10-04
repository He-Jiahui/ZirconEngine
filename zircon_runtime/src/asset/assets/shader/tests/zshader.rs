use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 21;
const LOOKUPS_PER_SAMPLE: usize = 40_000;

#[test]
fn borrowed_shader_token_contract_zshader() {
    let mut table = toml::Table::new();
    table.insert(
        "kind".to_string(),
        toml::Value::String("  SuRfAcE  ".to_string()),
    );
    assert_eq!(zshader_v2_kind(&table), Ok(ShaderAssetKind::Surface));
    assert!(stage_is_compute("  Cs "));
    assert!(stage_is_fragment(" FrAg "));
    assert!(!stage_is_compute("fragment"));
    assert_eq!(
        shader_asset_kind_from_token(" UnKnOwN "),
        None,
        "unknown tokens remain unsupported"
    );

    table.insert(
        "kind".to_string(),
        toml::Value::String(" UnKnOwN ".to_string()),
    );
    assert_eq!(
        zshader_v2_kind(&table),
        Err(ZShaderV2Error::UnsupportedKind {
            kind: "unknown".to_string(),
        })
    );
}

#[test]
#[ignore = "release performance gate"]
fn borrowed_shader_token_performance_release_zshader() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair_index in 0..SAMPLE_PAIRS {
        let (legacy_ns, optimized_ns) = if pair_index % 2 == 0 {
            (measure_legacy_tokens(), measure_borrowed_tokens())
        } else {
            let optimized_ns = measure_borrowed_tokens();
            (measure_legacy_tokens(), optimized_ns)
        };
        legacy_samples.push(legacy_ns);
        optimized_samples.push(optimized_ns);
    }

    let legacy_p95 = nearest_rank_p95(&legacy_samples);
    let optimized_p95 = nearest_rank_p95(&optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "PERF_RESULT plugins07_zshader_token_dispatch sample_pairs={SAMPLE_PAIRS} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25 legacy_allocations_per_sample={} optimized_allocations_per_sample=0 order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10",
        csv(&legacy_samples),
        csv(&optimized_samples),
        LOOKUPS_PER_SAMPLE * 6,
    );
    assert!(
        improvement_percent >= 25,
        "borrowed zshader token matching must improve P95 by at least 25%"
    );
}

fn measure_legacy_tokens() -> u128 {
    let started = Instant::now();
    let mut matched = 0_u64;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        for token in [" Surface ", "COMPUTE", "fullscreen"] {
            matched += u64::from(legacy_kind(black_box(token)).is_some());
        }
        matched += u64::from(legacy_stage_is_compute(black_box(" Comp ")));
        matched += u64::from(legacy_stage_is_fragment(black_box("FRAG")));
        matched += u64::from(legacy_stage_is_fragment(black_box(" fs ")));
    }
    black_box(matched);
    started.elapsed().as_nanos()
}

fn measure_borrowed_tokens() -> u128 {
    let started = Instant::now();
    let mut matched = 0_u64;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        for token in [" Surface ", "COMPUTE", "fullscreen"] {
            matched += u64::from(shader_asset_kind_from_token(black_box(token)).is_some());
        }
        matched += u64::from(stage_is_compute(black_box(" Comp ")));
        matched += u64::from(stage_is_fragment(black_box("FRAG")));
        matched += u64::from(stage_is_fragment(black_box(" fs ")));
    }
    black_box(matched);
    started.elapsed().as_nanos()
}

fn legacy_kind(token: &str) -> Option<ShaderAssetKind> {
    match token.trim().to_ascii_lowercase().as_str() {
        "surface" => Some(ShaderAssetKind::Surface),
        "include" => Some(ShaderAssetKind::Include),
        "compute" => Some(ShaderAssetKind::Compute),
        "fullscreen" => Some(ShaderAssetKind::Fullscreen),
        _ => None,
    }
}

fn legacy_stage_is_compute(token: &str) -> bool {
    matches!(
        token.trim().to_ascii_lowercase().as_str(),
        "compute" | "comp" | "cs"
    )
}

fn legacy_stage_is_fragment(token: &str) -> bool {
    matches!(
        token.trim().to_ascii_lowercase().as_str(),
        "fragment" | "frag" | "fs"
    )
}

fn nearest_rank_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * 95).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
