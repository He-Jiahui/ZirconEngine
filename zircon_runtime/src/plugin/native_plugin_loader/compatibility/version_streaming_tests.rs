use std::hint::black_box;
use std::time::Instant;

use super::{
    parse_engine_version, parse_version_component, EngineVersion,
    NativeDistributionCompatibilityError, NativeDistributionCompatibilityResult,
};

const MARKER: &str = "RUNTIME241_NATIVE_VERSION_STREAMING_PARSE_BENCH_V1";
const SAMPLE_PAIRS: usize = 17;
const REPEATS: usize = 262_144;

#[test]
fn optimization_batch_20260826gu_runtime241_version_parser_preserves_shape_and_error_precedence() {
    assert_eq!(
        parse_engine_version("12.34").unwrap(),
        EngineVersion {
            major: 12,
            minor: 34,
            patch: 0,
        }
    );
    assert_eq!(
        parse_engine_version("12.34.56-preview+build").unwrap(),
        EngineVersion {
            major: 12,
            minor: 34,
            patch: 56,
        }
    );
    assert!(matches!(
        parse_engine_version("invalid.2.3.4"),
        Err(NativeDistributionCompatibilityError::InvalidVersionShape { .. })
    ));
    assert!(matches!(
        parse_engine_version("1.2.invalid"),
        Err(NativeDistributionCompatibilityError::NonNumericVersionComponent { .. })
    ));
}

#[test]
fn optimization_batch_20260826gu_runtime241_version_parser_streams_fixed_segments() {
    let source = include_str!("../compatibility.rs");
    let implementation = source
        .split("fn parse_engine_version")
        .nth(1)
        .and_then(|tail| tail.split("fn parse_version_component").next())
        .expect("version parser implementation");
    assert!(implementation.contains("let mut parts = release.split('.')"));
    assert!(implementation.contains("let extra = parts.next()"));
    assert!(!implementation.contains("collect::<Vec<_>>()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gu_runtime241_native_version_streaming_parse_bench() {
    let version = "12.34.56-preview+build";
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(version, legacy_parse_engine_version));
            optimized_samples.push(measure(version, parse_engine_version));
        } else {
            optimized_samples.push(measure(version, parse_engine_version));
            legacy_samples.push(measure(version, legacy_parse_engine_version));
        }
    }

    let legacy_p95_ns = p95(&mut legacy_samples);
    let optimized_p95_ns = p95(&mut optimized_samples);
    println!("{MARKER} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns}");
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "streaming version parsing must use at most 70% of legacy p95: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn legacy_parse_engine_version(
    version: &str,
) -> NativeDistributionCompatibilityResult<EngineVersion> {
    let release = version
        .split(|ch| ch == '-' || ch == '+')
        .next()
        .unwrap_or_default()
        .trim();
    if release.is_empty() {
        return Err(NativeDistributionCompatibilityError::EmptyVersion);
    }
    let parts = release.split('.').collect::<Vec<_>>();
    if parts.len() < 2 || parts.len() > 3 {
        return Err(NativeDistributionCompatibilityError::InvalidVersionShape {
            version: version.to_string(),
        });
    }
    let major = parse_version_component(parts[0], version)?;
    let minor = parse_version_component(parts[1], version)?;
    let patch = if parts.len() == 3 {
        parse_version_component(parts[2], version)?
    } else {
        0
    };
    Ok(EngineVersion {
        major,
        minor,
        patch,
    })
}

fn measure(
    version: &str,
    implementation: fn(&str) -> NativeDistributionCompatibilityResult<EngineVersion>,
) -> u64 {
    let started = Instant::now();
    let mut checksum = 0u64;
    for _ in 0..REPEATS {
        let parsed = implementation(black_box(version)).expect("benchmark version");
        checksum = checksum.wrapping_add(parsed.major + parsed.minor + parsed.patch);
    }
    black_box(checksum);
    started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64
}

fn p95(samples: &mut [u64]) -> u64 {
    samples.sort_unstable();
    let index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
    samples[index]
}
