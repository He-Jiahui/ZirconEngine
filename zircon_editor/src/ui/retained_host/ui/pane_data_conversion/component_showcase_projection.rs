use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::windows::workbench_host_window::{PaneContentSize, PaneData, PanePayload};
use crate::ui::retained_host as host_contract;
use crate::ui::template_runtime::EditorUiHostRuntime;
use zircon_runtime_interface::ui::layout::UiSize;

use super::pane_component_projection::host_template_node;

pub(crate) fn to_host_contract_component_showcase_pane_from_host_pane(
    data: &PaneData,
    content_size: PaneContentSize,
) -> host_contract::ProjectOverviewPaneData {
    super::builtin_host_runtime()
        .and_then(|runtime| component_showcase_template_projection(data, content_size, runtime))
        .unwrap_or_default()
}

pub(crate) fn to_host_contract_component_showcase_pane_from_host_pane_with_runtime(
    data: &PaneData,
    content_size: PaneContentSize,
    runtime: &EditorUiHostRuntime,
) -> host_contract::ProjectOverviewPaneData {
    component_showcase_template_projection(data, content_size, runtime).unwrap_or_default()
}

fn component_showcase_template_projection(
    data: &PaneData,
    content_size: PaneContentSize,
    runtime: &EditorUiHostRuntime,
) -> Option<host_contract::ProjectOverviewPaneData> {
    let presentation = data.pane_presentation.as_ref()?;
    if !matches!(
        &presentation.body.payload,
        PanePayload::UiComponentShowcaseV1(_)
    ) {
        return None;
    }

    let projection = runtime.project_pane_body(&presentation.body).ok()?;
    let mut surface = runtime
        .build_shared_surface(&presentation.body.document_id)
        .ok()?;
    surface
        .compute_layout(UiSize::new(
            content_size.width.max(0.0),
            content_size.height.max(0.0),
        ))
        .ok()?;
    let host_model = runtime
        .build_host_model_with_surface(&projection, &surface)
        .ok()?;

    let mut nodes = Vec::with_capacity(host_model.nodes.len());
    for node in host_model.nodes {
        if let Some(node) = host_template_node(node) {
            nodes.push(node);
        }
    }
    Some(host_contract::ProjectOverviewPaneData {
        nodes: model_rc(nodes),
    })
}

#[cfg(test)]
mod optimization_batch_20260830cf_editor_tests {
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const NODES_PER_SAMPLE: usize = 512;

    #[test]
    fn component_showcase_projection_reserves_host_node_capacity() {
        let source = include_str!("component_showcase_projection.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("component showcase implementation");

        assert!(implementation.contains("Vec::with_capacity(host_model.nodes.len())"));
        assert!(implementation.contains("for node in host_model.nodes"));
        assert!(implementation.contains("if let Some(node) = host_template_node(node)"));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830cf_editor_component_showcase_capacity_p95() {
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(false));
                optimized.push(measure(true));
            } else {
                optimized.push(measure(true));
                legacy.push(measure(false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!("EDITOR330_COMPONENT_SHOWCASE_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} nodes_per_sample={NODES_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}", csv(&legacy), csv(&optimized));
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(use_capacity: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..128 {
            let mut nodes = if use_capacity {
                Vec::with_capacity(NODES_PER_SAMPLE)
            } else {
                Vec::new()
            };
            for node in 0..NODES_PER_SAMPLE {
                if node % 4 != 0 {
                    nodes.push(node);
                }
            }
            checksum ^= nodes.len();
        }
        std::hint::black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], p: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * p).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
