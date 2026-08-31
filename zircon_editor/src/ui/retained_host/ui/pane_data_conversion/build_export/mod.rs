mod target_rows;

#[cfg(test)]
mod tests;

use self::target_rows::build_export_target_row_nodes;
use super::build_export_wizard_panel::{
    build_export_pane_supports_wizard_projection, build_export_wizard_panel_nodes,
};
use super::model_projection::map_model_rc;
use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::windows::workbench_host_window::{
    BuildExportTargetViewData, PaneContentSize, PaneData,
};
use crate::ui::retained_host as host_contract;

pub(crate) fn to_host_contract_build_export_pane_from_host_pane(
    data: &PaneData,
    content_size: PaneContentSize,
) -> host_contract::BuildExportPaneData {
    let native = &data.native_body.build_export;
    let nodes = if build_export_pane_supports_wizard_projection(data) {
        build_export_wizard_panel_nodes(native, content_size).unwrap_or_else(|| {
            let nodes = build_export_template_projection(data, content_size).unwrap_or_default();
            let target_rows = build_export_target_row_nodes(native, &nodes, content_size);
            append_or_adopt_target_rows(nodes, target_rows)
        })
    } else {
        let nodes = build_export_template_projection(data, content_size).unwrap_or_default();
        let target_rows = build_export_target_row_nodes(native, &nodes, content_size);
        append_or_adopt_target_rows(nodes, target_rows)
    };

    host_contract::BuildExportPaneData {
        nodes: model_rc(nodes),
        targets: map_model_rc(&native.targets, to_host_contract_build_export_target),
        diagnostics: native.diagnostics.clone(),
    }
}

fn append_or_adopt_target_rows<T>(mut nodes: Vec<T>, mut target_rows: Vec<T>) -> Vec<T> {
    if nodes.is_empty() && nodes.capacity() == 0 {
        return target_rows;
    }
    nodes.append(&mut target_rows);
    nodes
}

fn to_host_contract_build_export_target(
    data: &BuildExportTargetViewData,
) -> host_contract::BuildExportTargetData {
    host_contract::BuildExportTargetData {
        profile_name: data.profile_name.clone(),
        platform: data.platform.clone(),
        target_mode: data.target_mode.clone(),
        strategies: data.strategies.clone(),
        status: data.status.clone(),
        enabled_plugins: data.enabled_plugins.clone(),
        linked_runtime_crates: data.linked_runtime_crates.clone(),
        native_dynamic_packages: data.native_dynamic_packages.clone(),
        generated_files: data.generated_files.clone(),
        diagnostics: data.diagnostics.clone(),
        fatal: data.fatal,
    }
}

fn build_export_template_projection(
    data: &PaneData,
    content_size: PaneContentSize,
) -> Option<Vec<host_contract::TemplatePaneNodeData>> {
    let presentation = data.pane_presentation.as_ref()?;
    if !matches!(
        &presentation.body.payload,
        crate::ui::layouts::windows::workbench_host_window::PanePayload::BuildExportV1(_)
    ) {
        return None;
    }

    super::project_pane_template_nodes(&presentation.body, content_size)
}

#[cfg(test)]
mod optimization_batch_tests {
    use std::hint::black_box;
    use std::time::Instant;

    #[test]
    fn optimization_batch_dr_build_export_moves_owned_target_rows() {
        let source = include_str!("mod.rs");
        let start = source
            .find("pub(crate) fn to_host_contract_build_export_pane_from_host_pane")
            .expect("build export projection start");
        let end = source[start..]
            .find("fn to_host_contract_build_export_target")
            .map(|offset| start + offset)
            .expect("build export projection end");
        let production = &source[start..end];
        assert!(production.contains("return target_rows"));
        assert!(production.contains("nodes.append(&mut target_rows)"));
        assert!(!production.contains("nodes.extend(build_export_target_row_nodes"));
    }

    #[test]
    #[ignore = "release-only alternating p95 performance gate"]
    fn optimization_batch_dr_build_export_owned_target_rows_p95() {
        const SAMPLE_PAIRS: usize = 17;
        const BUILDS_PER_SAMPLE: usize = 8_192;
        const ROWS_PER_BUILD: usize = 1_024;

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_owned_row_merge(
                    BUILDS_PER_SAMPLE,
                    ROWS_PER_BUILD,
                    false,
                ));
                optimized_samples.push(measure_owned_row_merge(
                    BUILDS_PER_SAMPLE,
                    ROWS_PER_BUILD,
                    true,
                ));
            } else {
                optimized_samples.push(measure_owned_row_merge(
                    BUILDS_PER_SAMPLE,
                    ROWS_PER_BUILD,
                    true,
                ));
                legacy_samples.push(measure_owned_row_merge(
                    BUILDS_PER_SAMPLE,
                    ROWS_PER_BUILD,
                    false,
                ));
            }
        }

        let legacy_p95 = p95(&mut legacy_samples);
        let optimized_p95 = p95(&mut optimized_samples);
        println!(
            "EDITOR354_BUILD_EXPORT_OWNED_TARGET_ROWS_BENCH_V1 builds_per_sample={BUILDS_PER_SAMPLE} rows_per_build={ROWS_PER_BUILD} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
            optimized_p95 as f64 / legacy_p95.max(1) as f64
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
            "build export owned target rows p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
        );

        fn measure_owned_row_merge(build_count: usize, row_count: usize, append: bool) -> u128 {
            let started_at = Instant::now();
            let mut checksum = 0_usize;
            for build_index in 0..build_count {
                let nodes = Vec::new();
                let target_rows = (0..row_count)
                    .map(|row| row ^ build_index)
                    .collect::<Vec<_>>();
                let nodes = if append {
                    super::append_or_adopt_target_rows(nodes, target_rows)
                } else {
                    let mut nodes = nodes;
                    nodes.extend(target_rows);
                    nodes
                };
                checksum = checksum.wrapping_add(nodes.len() ^ nodes.capacity());
                black_box(&nodes);
            }
            black_box(checksum);
            started_at.elapsed().as_nanos()
        }

        fn p95(samples: &mut [u128]) -> u128 {
            samples.sort_unstable();
            samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
        }
    }
}
