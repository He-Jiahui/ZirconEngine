use std::hint::black_box;
use std::time::Instant;

use super::project_selection_from_package;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::plugin::{PluginModuleManifest, PluginPackageManifest};

const SAMPLE_PAIRS: usize = 21;
const PACKAGES_PER_SAMPLE: usize = 512;
const MODES_PER_PACKAGE: usize = 4_096;

#[test]
fn optimization_batch_20260826gi_editor175_bitset_preserves_first_target_mode_order() {
    let package = PluginPackageManifest::new("editor_tools", "Editor Tools")
        .with_runtime_module(
            PluginModuleManifest::runtime("editor_tools.runtime", "editor_tools_runtime")
                .with_target_modes([
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::ServerRuntime,
                    RuntimeTargetMode::ClientRuntime,
                ]),
        )
        .with_editor_module(
            PluginModuleManifest::editor("editor_tools.editor", "editor_tools_editor")
                .with_target_modes([
                    RuntimeTargetMode::ServerRuntime,
                    RuntimeTargetMode::EditorHost,
                ]),
        );

    assert_eq!(
        project_selection_from_package(&package).target_modes,
        vec![
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::ServerRuntime,
            RuntimeTargetMode::EditorHost,
        ]
    );
}

#[test]
fn optimization_batch_20260826gi_editor175_package_projection_uses_target_mode_bits() {
    let source = include_str!("../project_selection.rs");

    assert!(source.contains("let mut seen_target_modes = 0_u8;"));
    assert!(source.contains("let bit = package_target_mode_bit(target_mode);"));
    assert!(source.contains("if seen_target_modes & bit == 0"));
    assert!(source.contains("seen_target_modes |= bit;"));
    assert!(!source.contains("target_modes.contains(&target_mode)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gi_editor175_package_target_mode_bitset_bench() {
    let modes = (0..MODES_PER_PACKAGE)
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
        "EDITOR175_PACKAGE_TARGET_MODE_BITSET_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
packages_per_sample={PACKAGES_PER_SAMPLE} modes_per_package={MODES_PER_PACKAGE} \
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
    for package in 0..PACKAGES_PER_SAMPLE {
        let modes = black_box(modes);
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
            checksum ^= black_box(seen_target_modes as usize);
        } else {
            for mode in modes.iter().copied() {
                if !target_modes.contains(&mode) {
                    target_modes.push(mode);
                }
            }
        }
        checksum ^= black_box(target_modes.len() ^ target_modes.capacity() ^ package);
        black_box(&target_modes);
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
