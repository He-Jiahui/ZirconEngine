mod actions;
mod identity;
mod metrics;
mod node;
mod row;

use std::collections::HashMap;

use crate::ui::layouts::windows::workbench_host_window::{
    BuildExportPaneViewData, PaneContentSize,
};
use crate::ui::retained_host as host_contract;

use self::identity::{build_export_key, build_export_target_id};
use self::metrics::{BUILD_EXPORT_NODES_PER_TARGET, BUILD_EXPORT_ROW_GAP, BUILD_EXPORT_ROW_HEIGHT};
use self::row::build_export_target_nodes;

pub(super) fn build_export_target_row_nodes(
    data: &BuildExportPaneViewData,
    template_nodes: &[host_contract::TemplatePaneNodeData],
    content_size: PaneContentSize,
) -> Vec<host_contract::TemplatePaneNodeData> {
    let list_frame = build_export_target_list_frame(template_nodes, content_size);
    let list_width = list_frame.width.max(content_size.width).max(0.0);
    let targets_with_platform_id = data
        .targets
        .iter()
        .map(|target| {
            let platform_id = build_export_key(target.platform.as_str());
            (target, platform_id)
        })
        .collect::<Vec<_>>();
    let mut platform_counts = HashMap::new();
    for (_, platform_id) in &targets_with_platform_id {
        *platform_counts
            .entry(platform_id.as_str())
            .or_insert(0usize) += 1;
    }
    let target_ids = targets_with_platform_id
        .iter()
        .map(|(target, platform_id)| {
            build_export_target_id(
                platform_id,
                target.profile_name.as_str(),
                platform_counts
                    .get(platform_id.as_str())
                    .copied()
                    .unwrap_or(0)
                    > 1,
            )
        })
        .collect::<Vec<_>>();
    drop(platform_counts);
    let mut target_id_counts = HashMap::new();
    for target_id in &target_ids {
        *target_id_counts.entry(target_id.clone()).or_insert(0usize) += 1;
    }
    let mut target_id_occurrences = HashMap::new();
    let mut nodes = Vec::with_capacity(
        targets_with_platform_id
            .len()
            .saturating_mul(BUILD_EXPORT_NODES_PER_TARGET),
    );

    for (row, ((target, _), mut target_id)) in targets_with_platform_id
        .into_iter()
        .zip(target_ids)
        .enumerate()
    {
        if target_id_counts.get(&target_id).copied().unwrap_or(0) > 1 {
            let occurrence = target_id_occurrences
                .entry(target_id.clone())
                .and_modify(|count| *count += 1)
                .or_insert(1usize);
            target_id = format!("{target_id}.{occurrence}");
        }
        nodes.extend(build_export_target_nodes(
            row,
            &target_id,
            target,
            &list_frame,
            list_width,
        ));
    }

    nodes
}

fn build_export_target_list_frame(
    template_nodes: &[host_contract::TemplatePaneNodeData],
    content_size: PaneContentSize,
) -> host_contract::TemplateNodeFrameData {
    template_nodes
        .iter()
        .find(|node| {
            matches!(
                node.control_id.as_str(),
                "BuildExportTargetsSlotAnchor" | "BuildExportTargetsPanel"
            )
        })
        .map(|node| node.frame.clone())
        .unwrap_or_else(|| host_contract::TemplateNodeFrameData {
            x: 0.0,
            y: 0.0,
            width: content_size.width.max(0.0),
            height: content_size.height.max(0.0),
        })
}
