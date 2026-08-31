use std::hint::black_box;
use std::time::Instant;

use super::native_package_target_modes;
use crate::core::framework::platform::RuntimeTargetMode;
use crate::plugin::{PluginModuleManifest, PluginPackageManifest};

const SAMPLE_PAIRS: usize = 21;
const PROJECTIONS_PER_SAMPLE: usize = 512;
const MODES_PER_PROJECTION: usize = 4_096;

#[test]
fn optimization_batch_20260826gj_runtime230_bitset_preserves_runtime_mode_filter_and_order() {
    let package = PluginPackageManifest::new("native_rendering", "Native Rendering")
        .with_runtime_module(
            PluginModuleManifest::runtime("native_rendering.client", "native_rendering_client")
                .with_target_modes([
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::ServerRuntime,
                    RuntimeTargetMode::ClientRuntime,
                ]),
        )
        .with_runtime_module(
            PluginModuleManifest::runtime("native_rendering.server", "native_rendering_server")
                .with_target_modes([RuntimeTargetMode::ServerRuntime]),
        )
        .with_editor_module(
            PluginModuleManifest::editor("native_rendering.editor", "native_rendering_editor")
                .with_target_modes([RuntimeTargetMode::EditorHost]),
        );

    assert_eq!(
        native_package_target_modes(&package),
        vec![
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::ServerRuntime,
        ]
    );
}

#[test]
fn optimization_batch_20260826gj_runtime230_native_projection_uses_target_mode_bits() {
    let source = include_str!("../target_modes.rs");

    assert!(source.contains("let mut seen_target_modes = 0_u8;"));
    assert!(source.contains("let bit = native_package_target_mode_bit(target_mode);"));
    assert!(source.contains("if seen_target_modes & bit == 0"));
    assert!(source.contains("module.kind == PluginModuleKind::Runtime"));
    assert!(!source.contains("target_modes.contains(&target_mode)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gj_runtime230_native_package_target_mode_bitset_bench() {
    let modes = (0..MODES_PER_PROJECTION)
        .map(|index| (index % 3) as u8)
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&modes, false));
            optimized_samples.push(measure(&modes, true));
        } else {
            optimized_samples.push(measure(&modes, true));
            legacy_samples.push(measure(&modes, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME230_NATIVE_PACKAGE_TARGET_MODE_BITSET_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
projections_per_sample={PROJECTIONS_PER_SAMPLE} modes_per_projection={MODES_PER_PROJECTION} \
distinct_target_modes=3 legacy_membership=linear_vec optimized_membership=three_bit_mask \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(modes: &[u8], use_bitset: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for projection in 0..PROJECTIONS_PER_SAMPLE {
        let target_modes = project_modes(black_box(modes), use_bitset);
        checksum ^= black_box(target_modes.len() ^ target_modes.capacity() ^ projection);
        black_box(&target_modes);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn project_modes(modes: &[u8], use_bitset: bool) -> Vec<u8> {
    let mut target_modes = Vec::new();
    if use_bitset {
        let mut seen_target_modes = 0_u8;
        for mode in modes.iter().copied() {
            let bit = 1_u8 << mode;
            if seen_target_modes & bit == 0 {
                seen_target_modes |= bit;
                target_modes.push(mode);
            }
        }
        black_box(seen_target_modes);
    } else {
        for mode in modes.iter().copied() {
            if !target_modes.contains(&mode) {
                target_modes.push(mode);
            }
        }
    }
    target_modes
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
