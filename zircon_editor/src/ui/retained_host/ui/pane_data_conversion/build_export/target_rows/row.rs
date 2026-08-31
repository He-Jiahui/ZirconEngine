use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::windows::workbench_host_window::BuildExportTargetViewData;
use crate::ui::retained_host as host_contract;

use super::actions::{build_export_action_button_nodes, build_export_row_actions};
use super::metrics::{
    BUILD_EXPORT_NODES_PER_TARGET, BUILD_EXPORT_ROW_GAP, BUILD_EXPORT_ROW_HEIGHT,
    BUILD_EXPORT_ROW_PADDING,
};
use super::node::build_export_node;

pub(super) fn build_export_target_nodes(
    row: usize,
    target_id: &str,
    target: &BuildExportTargetViewData,
    list_frame: &host_contract::TemplateNodeFrameData,
    list_width: f32,
) -> Vec<host_contract::TemplatePaneNodeData> {
    let node_id = format!("{row}_{target_id}");
    let row_y = list_frame.y + row as f32 * (BUILD_EXPORT_ROW_HEIGHT + BUILD_EXPORT_ROW_GAP);
    let actions = build_export_row_actions(target);
    let mut nodes = Vec::with_capacity(BUILD_EXPORT_NODES_PER_TARGET);

    let mut row_node = build_export_node(
        format!("build_export_row_{node_id}"),
        format!("BuildExportRow.{target_id}"),
        "Panel",
        target.profile_name.to_string(),
        host_contract::TemplateNodeFrameData {
            x: list_frame.x,
            y: row_y,
            width: list_width,
            height: BUILD_EXPORT_ROW_HEIGHT,
        },
    );
    row_node.surface_variant = if target.fatal {
        "diagnostic-error".into()
    } else {
        "build-export-row".into()
    };
    row_node.corner_radius = 6.0;
    row_node.border_width = 1.0;
    row_node.actions = model_rc(
        actions
            .iter()
            .map(|action| host_contract::TemplatePaneActionData {
                label: action.label.into(),
                action_id: action.action_id.clone().into(),
            })
            .collect(),
    );
    nodes.push(row_node);

    nodes.push(build_export_node(
        format!("build_export_title_{node_id}"),
        format!("BuildExportTitle.{target_id}"),
        "Label",
        format!("{} | {}", target.platform, target.status),
        host_contract::TemplateNodeFrameData {
            x: list_frame.x + BUILD_EXPORT_ROW_PADDING,
            y: row_y + 8.0,
            width: (list_width - BUILD_EXPORT_ROW_PADDING * 2.0).max(0.0),
            height: 20.0,
        },
    ));

    let mut strategy = build_export_node(
        format!("build_export_strategy_{node_id}"),
        format!("BuildExportStrategy.{target_id}"),
        "Label",
        format!("{} | {}", target.target_mode, target.strategies),
        host_contract::TemplateNodeFrameData {
            x: list_frame.x + BUILD_EXPORT_ROW_PADDING,
            y: row_y + 30.0,
            width: (list_width - BUILD_EXPORT_ROW_PADDING * 2.0).max(0.0),
            height: 18.0,
        },
    );
    strategy.text_tone = "muted".into();
    nodes.push(strategy);

    let mut counts = build_export_node(
        format!("build_export_counts_{node_id}"),
        format!("BuildExportCounts.{target_id}"),
        "Label",
        format!(
            "plugins {} | linked {} | native {} | files {}",
            target.enabled_plugins,
            target.linked_runtime_crates,
            target.native_dynamic_packages,
            target.generated_files
        ),
        host_contract::TemplateNodeFrameData {
            x: list_frame.x + BUILD_EXPORT_ROW_PADDING,
            y: row_y + 48.0,
            width: (list_width - BUILD_EXPORT_ROW_PADDING * 2.0).max(0.0),
            height: 18.0,
        },
    );
    counts.text_tone = "muted".into();
    nodes.push(counts);

    let mut diagnostics = build_export_node(
        format!("build_export_diagnostics_{node_id}"),
        format!("BuildExportDiagnostics.{target_id}"),
        "Label",
        target.diagnostics.to_string(),
        host_contract::TemplateNodeFrameData {
            x: list_frame.x + BUILD_EXPORT_ROW_PADDING,
            y: row_y + 66.0,
            width: (list_width - BUILD_EXPORT_ROW_PADDING * 2.0).max(0.0),
            height: 18.0,
        },
    );
    diagnostics.text_tone = if target.fatal { "danger" } else { "muted" }.into();
    nodes.push(diagnostics);

    nodes.extend(build_export_action_button_nodes(
        &node_id,
        row_y,
        list_frame.x,
        list_width,
        &actions,
    ));

    nodes
}
