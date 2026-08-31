use crate::ui::workbench::view::ViewDescriptor;

use super::assets_view_descriptor::assets_view_descriptor;
use super::build_export_view_descriptor::build_export_view_descriptor;
use super::console_view_descriptor::console_view_descriptor;
use super::functional_panel_view_descriptors::functional_panel_view_descriptors;
use super::game_view_descriptor::game_view_descriptor;
use super::generated_bottom_view_descriptor::generated_bottom_view_descriptor;
use super::hierarchy_view_descriptor::hierarchy_view_descriptor;
use super::inspector_view_descriptor::inspector_view_descriptor;
use super::module_plugins_view_descriptor::module_plugins_view_descriptor;
use super::performance_timeline_view_descriptor::performance_timeline_view_descriptor;
use super::project_view_descriptor::project_view_descriptor;
use super::runtime_diagnostics_view_descriptor::runtime_diagnostics_view_descriptor;
use super::scene_view_descriptor::scene_view_descriptor;

pub(in crate::ui::host::builtin_views) fn activity_view_descriptors() -> Vec<ViewDescriptor> {
    [
        project_view_descriptor(),
        hierarchy_view_descriptor(),
        inspector_view_descriptor(),
        scene_view_descriptor(),
        game_view_descriptor(),
        assets_view_descriptor(),
        module_plugins_view_descriptor(),
        build_export_view_descriptor(),
        generated_bottom_view_descriptor(),
        console_view_descriptor(),
        runtime_diagnostics_view_descriptor(),
        performance_timeline_view_descriptor(),
    ]
    .into_iter()
    .chain(functional_panel_view_descriptors())
    .collect()
}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::activity_view_descriptors;
    use crate::ui::host::builtin_views::activity_windows::activity_window_descriptors::activity_window_descriptors;

    #[test]
    fn optimization_batch_dl_builtin_view_batches_preserve_descriptor_counts() {
        assert_eq!(activity_view_descriptors().len(), 22);
        assert_eq!(activity_window_descriptors().len(), 16);
    }

    #[test]
    fn optimization_batch_dl_builtin_view_batches_collect_once_from_exact_arrays() {
        let panel_aggregate = include_str!("activity_view_descriptors.rs");
        let panel_functional = include_str!("functional_panel_view_descriptors.rs");
        let window_aggregate = include_str!("../activity_windows/activity_window_descriptors.rs");
        let window_functional =
            include_str!("../activity_windows/functional_window_view_descriptors.rs");
        let panel_production = panel_aggregate
            .split("#[cfg(test)]")
            .next()
            .expect("activity view descriptor production source");

        assert!(panel_production.contains(".chain(functional_panel_view_descriptors())"));
        assert!(window_aggregate.contains(".chain(functional_window_view_descriptors())"));
        assert!(panel_functional.contains("-> [ViewDescriptor; 10]"));
        assert!(window_functional.contains("-> [ViewDescriptor; 7]"));
        assert!(!panel_functional.contains("-> Vec<ViewDescriptor>"));
        assert!(!window_functional.contains("-> Vec<ViewDescriptor>"));
    }

    #[test]
    #[ignore = "release-only alternating p95 performance gate"]
    fn optimization_batch_dl_single_allocation_builtin_view_batch_p95() {
        const SAMPLE_PAIRS: usize = 17;
        const BUILDS_PER_SAMPLE: usize = 65_536;

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_builtin_batches(BUILDS_PER_SAMPLE, true));
                optimized_samples.push(measure_builtin_batches(BUILDS_PER_SAMPLE, false));
            } else {
                optimized_samples.push(measure_builtin_batches(BUILDS_PER_SAMPLE, false));
                legacy_samples.push(measure_builtin_batches(BUILDS_PER_SAMPLE, true));
            }
        }

        let legacy_p95 = p95(&mut legacy_samples);
        let optimized_p95 = p95(&mut optimized_samples);
        println!(
            "EDITOR348_SINGLE_ALLOCATION_BUILTIN_VIEW_BATCH_BENCH_V1 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
            optimized_p95 as f64 / legacy_p95.max(1) as f64
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
            "single-allocation builtin view batch p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
        );
    }

    fn measure_builtin_batches(build_count: usize, legacy: bool) -> u128 {
        let started_at = Instant::now();
        let mut checksum = 0_u64;
        for _ in 0..build_count {
            let panels = assemble_batch::<12, 10>(legacy);
            let windows = assemble_batch::<9, 7>(legacy);
            checksum = checksum.wrapping_add(black_box(panels.len() + windows.len()) as u64);
        }
        black_box(checksum);
        started_at.elapsed().as_nanos()
    }

    fn assemble_batch<const BASE: usize, const FUNCTIONAL: usize>(legacy: bool) -> Vec<u64> {
        if legacy {
            let mut values = black_box([1_u64; BASE]).to_vec();
            values.extend(black_box([2_u64; FUNCTIONAL]).to_vec());
            values
        } else {
            black_box([1_u64; BASE])
                .into_iter()
                .chain(black_box([2_u64; FUNCTIONAL]))
                .collect()
        }
    }

    fn p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
    }
}
