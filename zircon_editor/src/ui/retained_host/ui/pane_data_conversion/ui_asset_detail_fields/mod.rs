use crate::ui::asset_editor;
use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::ViewTemplateNodeData;
use crate::ui::retained_host as host_contract;
use crate::ui::retained_host::primitives::ModelRc;

use super::super::template_node_conversion::to_host_contract_template_node_owned;

mod binding;
mod layout;
mod row_model;
mod section_nodes;
mod sections;
mod slot;
mod widget;

pub(super) fn to_host_contract_ui_asset_template_nodes(
    items: Vec<ViewTemplateNodeData>,
    data: &asset_editor::UiAssetEditorPanePresentation,
    prop_state_rows: &[asset_editor::UiAssetEditorWidgetPropStateItem],
    instance_id: &str,
) -> ModelRc<host_contract::TemplatePaneNodeData> {
    let mut nodes = items
        .into_iter()
        .map(to_host_contract_template_node_owned)
        .collect::<Vec<_>>();
    let sections = sections::ui_asset_detail_field_sections(data, prop_state_rows);
    let additional_node_capacity = sections
        .iter()
        .map(|section| section.rows.len().saturating_mul(2))
        .sum();
    nodes.reserve(additional_node_capacity);
    for section in sections {
        section_nodes::append_detail_section_nodes(&mut nodes, &section, instance_id);
    }
    model_rc(nodes)
}

#[cfg(test)]
mod optimization_batch_20260830ck_editor_tests {
    use std::hint::black_box;
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const INITIAL_NODES: usize = 64;
    const DETAIL_SECTIONS: usize = 4;
    const ROWS_PER_SECTION: usize = 128;

    #[test]
    fn ui_asset_detail_projection_reserves_section_row_capacity() {
        let source = include_str!("mod.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("UI asset detail projection implementation");

        assert!(implementation.contains("let sections ="));
        assert!(implementation.contains("section.rows.len().saturating_mul(2)"));
        assert!(implementation.contains("nodes.reserve(additional_node_capacity)"));
        assert!(implementation.contains("for section in sections"));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830ck_editor_ui_asset_detail_capacity_p95() {
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
        println!("EDITOR333_UI_ASSET_DETAIL_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} initial_nodes={INITIAL_NODES} detail_sections={DETAIL_SECTIONS} rows_per_section={ROWS_PER_SECTION} nodes_per_row=2 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}", csv(&legacy), csv(&optimized));
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(use_capacity: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for iteration in 0..64_u64 {
            let mut nodes = (0..INITIAL_NODES)
                .map(|index| [index as u64; 8])
                .collect::<Vec<_>>();
            if use_capacity {
                nodes.reserve(DETAIL_SECTIONS * ROWS_PER_SECTION * 2);
            }
            for section in 0..DETAIL_SECTIONS {
                for row in 0..ROWS_PER_SECTION {
                    let value = iteration ^ (section * ROWS_PER_SECTION + row) as u64;
                    nodes.push([value; 8]);
                    nodes.push([value.wrapping_add(1); 8]);
                }
            }
            checksum ^= nodes.len();
            black_box(nodes);
        }
        black_box(checksum);
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
