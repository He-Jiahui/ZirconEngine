use super::ViewTemplateNodeData;
use crate::ui::retained_host::primitives::SharedString;
use zircon_runtime_interface::ui::design_tokens::EditorTypographyTokens;

const UTILITY_TAB_FONT_SIZE: f32 = EditorTypographyTokens::WORKBENCH_BODY_SIZE;
const UTILITY_TAB_SELECTED_FONT_WEIGHT: i32 = 600;
const UTILITY_TAB_IDLE_FONT_WEIGHT: i32 = 400;
const UTILITY_TAB_IDS: &[&str] = &[
    "AssetBrowserPreviewTabButton",
    "AssetBrowserReferencesTabButton",
    "AssetBrowserMetadataTabButton",
    "AssetBrowserPluginsTabButton",
];

pub(super) fn apply_asset_browser_utility_tab_typography(nodes: &mut [ViewTemplateNodeData]) {
    for node in nodes
        .iter_mut()
        .filter(|node| UTILITY_TAB_IDS.contains(&node.control_id.as_str()))
    {
        node.font_size = UTILITY_TAB_FONT_SIZE;
        node.font_weight = if node.selected {
            UTILITY_TAB_SELECTED_FONT_WEIGHT
        } else {
            UTILITY_TAB_IDLE_FONT_WEIGHT
        };
        assign_shared_string_if_changed(&mut node.overflow, "elide");
    }
}

fn assign_shared_string_if_changed(target: &mut SharedString, value: &str) {
    if target.as_str() != value {
        *target = value.into();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utility_tabs_share_workbench_body_typography_and_elide_overflow() {
        let mut nodes = vec![
            ViewTemplateNodeData {
                control_id: "AssetBrowserPreviewTabButton".into(),
                selected: true,
                ..ViewTemplateNodeData::default()
            },
            ViewTemplateNodeData {
                control_id: "AssetBrowserMetadataTabButton".into(),
                ..ViewTemplateNodeData::default()
            },
        ];

        apply_asset_browser_utility_tab_typography(&mut nodes);

        assert_eq!(
            nodes[0].font_size,
            EditorTypographyTokens::WORKBENCH_BODY_SIZE
        );
        assert_eq!(nodes[0].font_weight, UTILITY_TAB_SELECTED_FONT_WEIGHT);
        assert_eq!(nodes[1].font_weight, UTILITY_TAB_IDLE_FONT_WEIGHT);
        assert!(nodes.iter().all(|node| node.overflow == "elide"));
    }

    #[test]
    fn idempotent_utility_tab_overflow_preserves_storage() {
        let mut nodes = vec![utility_tab("AssetBrowserPreviewTabButton", true)];
        apply_asset_browser_utility_tab_typography(&mut nodes);
        let overflow_pointer = nodes[0].overflow.as_str().as_ptr();

        apply_asset_browser_utility_tab_typography(&mut nodes);

        assert_eq!(nodes[0].overflow.as_str().as_ptr(), overflow_pointer);
        assert_eq!(nodes[0].overflow.as_str(), "elide");
    }

    #[test]
    fn idempotent_utility_tab_overflow_uses_conditional_write() {
        let source = include_str!("utility_tabs.rs");
        let implementation = source.split("#[cfg(test)]").next().expect("implementation");

        assert!(implementation.contains("fn assign_shared_string_if_changed"));
        assert!(implementation.contains("assign_shared_string_if_changed(&mut node.overflow"));
        assert!(!implementation.contains("node.overflow = \"elide\".into()"));
    }

    #[test]
    #[ignore = "release performance benchmark"]
    fn idempotent_utility_tab_overflow_release_benchmark() {
        const SAMPLES: usize = 11;
        const ITERATIONS: usize = 16_384;
        const TAB_COUNT: usize = 4;
        const RETIRED_OVERFLOW_WRITES: usize = 4;
        const OPTIMIZED_OVERFLOW_WRITES: usize = 0;

        let base = UTILITY_TAB_IDS
            .iter()
            .enumerate()
            .map(|(index, control_id)| utility_tab(control_id, index == 0))
            .collect::<Vec<_>>();
        assert_eq!(base.len(), TAB_COUNT);
        let mut retired_samples = Vec::with_capacity(SAMPLES);
        let mut optimized_samples = Vec::with_capacity(SAMPLES);
        for sample in 0..SAMPLES {
            let benchmark = |apply: fn(&mut [ViewTemplateNodeData])| {
                let mut nodes = base.clone();
                apply(&mut nodes);
                let started = std::time::Instant::now();
                for _ in 0..ITERATIONS {
                    apply(&mut nodes);
                    for node in &nodes {
                        std::hint::black_box(node.overflow.as_str().as_ptr());
                    }
                }
                started.elapsed().as_nanos()
            };

            if sample % 2 == 0 {
                retired_samples.push(benchmark(retired_apply_utility_tab_typography));
                optimized_samples.push(benchmark(apply_asset_browser_utility_tab_typography));
            } else {
                optimized_samples.push(benchmark(apply_asset_browser_utility_tab_typography));
                retired_samples.push(benchmark(retired_apply_utility_tab_typography));
            }
        }

        let retired_p95_ns = percentile_95(&mut retired_samples);
        let optimized_p95_ns = percentile_95(&mut optimized_samples);
        let reduction_bps = retired_p95_ns
            .saturating_sub(optimized_p95_ns)
            .saturating_mul(10_000)
            / retired_p95_ns.max(1);
        println!(
            "EDITOR57_IDEMPOTENT_UTILITY_TAB_OVERFLOW_BENCH_V1 \
             retired_p95_ns={retired_p95_ns} optimized_p95_ns={optimized_p95_ns} \
             reduction_bps={reduction_bps} samples={SAMPLES} iterations={ITERATIONS} \
             tabs={TAB_COUNT} repeated_overflow_writes={RETIRED_OVERFLOW_WRITES}->{OPTIMIZED_OVERFLOW_WRITES}"
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= retired_p95_ns.saturating_mul(45),
            "optimized P95 must be at least 55% faster: retired={retired_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    fn retired_apply_utility_tab_typography(nodes: &mut [ViewTemplateNodeData]) {
        for node in nodes
            .iter_mut()
            .filter(|node| UTILITY_TAB_IDS.contains(&node.control_id.as_str()))
        {
            node.font_size = UTILITY_TAB_FONT_SIZE;
            node.font_weight = if node.selected {
                UTILITY_TAB_SELECTED_FONT_WEIGHT
            } else {
                UTILITY_TAB_IDLE_FONT_WEIGHT
            };
            node.overflow = "elide".into();
        }
    }

    fn utility_tab(control_id: &str, selected: bool) -> ViewTemplateNodeData {
        ViewTemplateNodeData {
            control_id: control_id.into(),
            selected,
            ..ViewTemplateNodeData::default()
        }
    }

    fn percentile_95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        let index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
        samples[index]
    }
}
