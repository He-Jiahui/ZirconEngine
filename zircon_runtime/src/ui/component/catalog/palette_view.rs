use serde::{Deserialize, Serialize};

use super::registry::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::component::{
    UiComponentCategory, UiDefaultNodeTemplate, UiHostCapabilitySet,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiComponentPaletteEntry {
    pub component_id: String,
    pub display_name: String,
    pub category: UiComponentCategory,
    #[serde(default)]
    pub icon: Option<String>,
    pub sort_key: String,
    pub default_node: UiDefaultNodeTemplate,
}

pub(super) fn palette_entries_for_host(
    registry: &UiComponentDescriptorRegistry,
    host_capabilities: &UiHostCapabilitySet,
) -> Vec<UiComponentPaletteEntry> {
    let mut entries = Vec::with_capacity(registry.len());
    entries.extend(
        registry
            .descriptors()
            .filter(|descriptor| {
                host_capabilities.contains_all(&descriptor.required_host_capabilities)
            })
            .filter_map(|descriptor| {
                let metadata = descriptor.palette.as_ref()?;
                Some(UiComponentPaletteEntry {
                    component_id: descriptor.id.clone(),
                    display_name: metadata.display_name.clone(),
                    category: metadata.category,
                    icon: metadata.icon.clone(),
                    sort_key: metadata.sort_key.clone(),
                    default_node: metadata.default_node.clone(),
                })
            }),
    );
    entries.sort_unstable_by(|left, right| {
        left.category
            .cmp(&right.category)
            .then_with(|| left.sort_key.cmp(&right.sort_key))
            .then_with(|| left.display_name.cmp(&right.display_name))
            .then_with(|| left.component_id.cmp(&right.component_id))
    });
    entries
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::{palette_entries_for_host, UiComponentDescriptorRegistry, UiHostCapabilitySet};

    #[derive(Clone)]
    struct PaletteSortRow {
        category: u8,
        sort_key: String,
        display_name: String,
        component_id: String,
    }

    #[test]
    fn preallocated_palette_projection_reserves_registry_upper_bound() {
        let registry = UiComponentDescriptorRegistry::editor_showcase_shared();
        let entries = palette_entries_for_host(registry, &UiHostCapabilitySet::editor_authoring());

        assert!(!entries.is_empty());
        assert!(entries.len() <= registry.len());
        assert_eq!(entries.capacity(), registry.len());
    }

    #[test]
    fn preallocated_palette_projection_preserves_sort_order() {
        let registry = UiComponentDescriptorRegistry::editor_showcase_shared();
        let entries = palette_entries_for_host(registry, &UiHostCapabilitySet::editor_authoring());

        assert!(entries.windows(2).all(|window| {
            (
                window[0].category,
                &window[0].sort_key,
                &window[0].display_name,
                &window[0].component_id,
            ) <= (
                window[1].category,
                &window[1].sort_key,
                &window[1].display_name,
                &window[1].component_id,
            )
        }));
    }

    #[test]
    fn optimization_batch_ej_palette_projection_uses_total_key_unstable_sort() {
        let source = include_str!("palette_view.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("palette projection implementation");

        assert!(implementation.contains("entries.sort_unstable_by("));
        assert!(!implementation.contains("entries.sort_by("));
        assert!(implementation.contains("left.component_id.cmp(&right.component_id)"));
    }

    #[test]
    #[ignore = "release-only unstable palette sort benchmark"]
    fn optimization_batch_ej_unstable_palette_sort_release_benchmark_evidence() {
        const SAMPLE_PAIRS: usize = 17;
        const ROWS: usize = 4_096;
        const SORTS_PER_SAMPLE: usize = 8;

        fn compare(left: &PaletteSortRow, right: &PaletteSortRow) -> std::cmp::Ordering {
            left.category
                .cmp(&right.category)
                .then_with(|| left.sort_key.cmp(&right.sort_key))
                .then_with(|| left.display_name.cmp(&right.display_name))
                .then_with(|| left.component_id.cmp(&right.component_id))
        }

        fn measure_legacy(base: &[PaletteSortRow]) -> u128 {
            let mut batches = (0..SORTS_PER_SAMPLE)
                .map(|_| base.to_vec())
                .collect::<Vec<_>>();
            let started = Instant::now();
            for rows in &mut batches {
                rows.sort_by(compare);
            }
            black_box(batches);
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized(base: &[PaletteSortRow]) -> u128 {
            let mut batches = (0..SORTS_PER_SAMPLE)
                .map(|_| base.to_vec())
                .collect::<Vec<_>>();
            let started = Instant::now();
            for rows in &mut batches {
                rows.sort_unstable_by(compare);
            }
            black_box(batches);
            started.elapsed().as_nanos().max(1)
        }

        fn percentile(samples: &[u128], percentile: usize) -> u128 {
            let mut sorted = samples.to_vec();
            sorted.sort_unstable();
            let rank = (sorted.len() * percentile).div_ceil(100);
            sorted[rank.saturating_sub(1)]
        }

        fn raw(samples: &[u128]) -> String {
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        }

        let base = (0..ROWS)
            .map(|index| {
                let key = (index * 2_653 + 1_013) % ROWS;
                PaletteSortRow {
                    category: (key % 8) as u8,
                    sort_key: format!("{:02}.{}.{key:04}", key % 8, "section".repeat(4)),
                    display_name: format!("Component {} {key:04}", "Display ".repeat(4)),
                    component_id: format!("component.{}.{key:04}", "namespace.".repeat(4)),
                }
            })
            .collect::<Vec<_>>();
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample in 0..SAMPLE_PAIRS {
            if sample % 2 == 0 {
                legacy_samples.push(measure_legacy(&base));
                optimized_samples.push(measure_optimized(&base));
            } else {
                optimized_samples.push(measure_optimized(&base));
                legacy_samples.push(measure_legacy(&base));
            }
        }

        let legacy_p50_ns = percentile(&legacy_samples, 50);
        let optimized_p50_ns = percentile(&optimized_samples, 50);
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);
        println!(
            "RUNTIME444_UNSTABLE_PALETTE_SORT_BENCH_V1 sample_pairs={SAMPLE_PAIRS} rows={ROWS} \
             sorts_per_sample={SORTS_PER_SAMPLE} pair_order=alternating_legacy_even \
             comparator=category_sort_key_display_name_component_id unique_component_ids=true \
             legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
             legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
             legacy_raw_ns={} optimized_raw_ns={}",
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(90),
            "unstable total-key palette sort must reduce P95 by at least 10%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }
}
