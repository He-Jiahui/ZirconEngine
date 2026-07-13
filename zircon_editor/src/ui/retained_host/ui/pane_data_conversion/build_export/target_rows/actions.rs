use crate::ui::layouts::windows::workbench_host_window::BuildExportTargetViewData;
use crate::ui::retained_host as host_contract;

use super::metrics::{
    BUILD_EXPORT_BUTTON_GAP, BUILD_EXPORT_BUTTON_HEIGHT, BUILD_EXPORT_PRIMARY_BUTTON_WIDTH,
    BUILD_EXPORT_ROW_HEIGHT, BUILD_EXPORT_ROW_PADDING, BUILD_EXPORT_SECONDARY_BUTTON_WIDTH,
};
use super::node::build_export_node;

pub(super) struct BuildExportRowAction {
    pub(super) label: &'static str,
    pub(super) action_id: String,
    pub(super) variant: &'static str,
    pub(super) disabled: bool,
    pub(super) width: f32,
}

pub(super) fn build_export_row_actions(
    target: &BuildExportTargetViewData,
) -> Vec<BuildExportRowAction> {
    let export_busy = matches!(
        target.status.as_str(),
        "Queued" | "Running" | "Cancel requested"
    );
    let (primary_label, primary_action_id) = if export_busy {
        (
            "Cancel",
            format!("workbench.build_export.cancel.{}", target.preset_name),
        )
    } else {
        (
            "Export",
            format!("workbench.build_export.execute.{}", target.preset_name),
        )
    };

    vec![
        BuildExportRowAction {
            label: primary_label,
            action_id: primary_action_id,
            variant: "primary",
            disabled: target.fatal && !export_busy,
            width: BUILD_EXPORT_PRIMARY_BUTTON_WIDTH,
        },
        BuildExportRowAction {
            label: "Choose",
            action_id: format!(
                "workbench.build_export.output.choose.{}",
                target.preset_name
            ),
            variant: "secondary",
            disabled: false,
            width: BUILD_EXPORT_SECONDARY_BUTTON_WIDTH,
        },
        BuildExportRowAction {
            label: "Open",
            action_id: format!(
                "workbench.build_export.output.reveal.{}",
                target.preset_name
            ),
            variant: "secondary",
            disabled: false,
            width: BUILD_EXPORT_SECONDARY_BUTTON_WIDTH,
        },
        BuildExportRowAction {
            label: "Default",
            action_id: format!("workbench.build_export.output.clear.{}", target.preset_name),
            variant: "secondary",
            disabled: false,
            width: BUILD_EXPORT_SECONDARY_BUTTON_WIDTH,
        },
    ]
}

pub(super) fn build_export_action_button_nodes(
    row_node_id: &str,
    row_y: f32,
    row_x: f32,
    row_width: f32,
    actions: &[BuildExportRowAction],
) -> Vec<host_contract::TemplatePaneNodeData> {
    if actions.is_empty() {
        return Vec::new();
    }

    let total_width = actions.iter().map(|action| action.width).sum::<f32>()
        + BUILD_EXPORT_BUTTON_GAP * actions.len().saturating_sub(1) as f32;
    let start_x = (row_x + row_width - BUILD_EXPORT_ROW_PADDING - total_width).max(row_x);
    let button_y =
        row_y + BUILD_EXPORT_ROW_HEIGHT - BUILD_EXPORT_ROW_PADDING - BUILD_EXPORT_BUTTON_HEIGHT;
    let mut cursor_x = start_x;

    actions
        .iter()
        .enumerate()
        .map(|(index, action)| {
            let mut node = build_export_node(
                format!("build_export_action_{row_node_id}_{index}"),
                "BuildExportAction",
                "Button",
                action.label,
                host_contract::TemplateNodeFrameData {
                    x: cursor_x,
                    y: button_y,
                    width: action.width,
                    height: BUILD_EXPORT_BUTTON_HEIGHT,
                },
            );
            cursor_x += action.width + BUILD_EXPORT_BUTTON_GAP;
            node.dispatch_kind = "build_export".into();
            node.action_id = action.action_id.clone().into();
            node.button_variant = action.variant.into();
            node.disabled = action.disabled;
            node
        })
        .collect()
}
