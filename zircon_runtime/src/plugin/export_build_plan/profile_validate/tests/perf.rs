use std::hint::black_box;
use std::time::Instant;

use super::{
    deduplicate_export_strategies, export_profile_strategy_diagnostics, ExportPackagingStrategy,
    ExportProfile,
};

const SAMPLE_PAIRS: usize = 17;
const STRATEGIES_PER_SAMPLE: usize = 65_536;

#[test]
fn optimization_batch_r6_wave_runtime633_preserves_duplicate_order_and_first_strategy() {
    use ExportPackagingStrategy::{LibraryEmbed, NativeDynamic, SourceTemplate};

    let mut strategies = vec![
        NativeDynamic,
        SourceTemplate,
        NativeDynamic,
        LibraryEmbed,
        SourceTemplate,
    ];
    deduplicate_export_strategies(&mut strategies);
    assert_eq!(strategies, [NativeDynamic, SourceTemplate, LibraryEmbed]);

    let profile = ExportProfile::default().with_strategies([
        NativeDynamic,
        SourceTemplate,
        NativeDynamic,
        LibraryEmbed,
        SourceTemplate,
    ]);
    let diagnostics = export_profile_strategy_diagnostics(&profile);
    assert_eq!(diagnostics.len(), 2);
    assert!(diagnostics[0].contains("NativeDynamic"));
    assert!(diagnostics[1].contains("SourceTemplate"));
}

#[test]
fn optimization_batch_r6_wave_runtime633_uses_stack_bitset_membership() {
    let source = include_str!("../../export_profile_validation.rs");
    assert_eq!(source.matches("let mut seen_mask = 0_u8;").count(), 2);
    assert!(source.contains("export_packaging_strategy_bit"));
    assert!(!source.contains("let mut seen = Vec::new();"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_r6_wave_runtime633_packaging_strategy_bitset_p95() {
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

    let legacy_p95 = p95(&legacy_samples);
    let optimized_p95 = p95(&optimized_samples);
    println!(
        "RUNTIME633_BITSET_EXPORT_PACKAGING_STRATEGY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} strategies_per_sample={STRATEGIES_PER_SAMPLE} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
        optimized_p95 as f64 / legacy_p95.max(1) as f64
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(85),
        "stack bitset packaging-strategy dedupe must be at least 15% faster at P95"
    );
}

fn measure(optimized: bool) -> u128 {
    use ExportPackagingStrategy::{LibraryEmbed, NativeDynamic, SourceTemplate};

    let mut strategies = (0..STRATEGIES_PER_SAMPLE)
        .map(|index| match index % 3 {
            0 => NativeDynamic,
            1 => SourceTemplate,
            _ => LibraryEmbed,
        })
        .collect::<Vec<_>>();
    let started = Instant::now();
    if optimized {
        deduplicate_export_strategies(black_box(&mut strategies));
    } else {
        legacy_deduplicate_export_strategies(black_box(&mut strategies));
    }
    let elapsed = started.elapsed().as_nanos().max(1);
    assert_eq!(strategies, [NativeDynamic, SourceTemplate, LibraryEmbed]);
    black_box(strategies.capacity());
    elapsed
}

fn legacy_deduplicate_export_strategies(strategies: &mut Vec<ExportPackagingStrategy>) {
    let mut seen = Vec::new();
    strategies.retain(|strategy| {
        if seen.contains(strategy) {
            false
        } else {
            seen.push(*strategy);
            true
        }
    });
}

fn p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * 95).div_ceil(100) - 1]
}
