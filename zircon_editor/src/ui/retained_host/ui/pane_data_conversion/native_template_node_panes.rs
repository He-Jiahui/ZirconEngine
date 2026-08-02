use crate::ui::layouts::windows::workbench_host_window::{
    AssetBrowserPaneViewData, AssetsActivityPaneViewData, PaneContentSize,
    ProjectOverviewPaneViewData,
};
use crate::ui::retained_host as host_contract;

use super::super::template_layout_context::apply_table_layout_context_variant;
use super::super::template_node_conversion::to_host_contract_template_node;
use super::template_node_projection::project_nodes;

pub(in super::super) fn to_host_contract_assets_activity_pane(
    data: AssetsActivityPaneViewData,
) -> host_contract::AssetsActivityPaneData {
    host_contract::AssetsActivityPaneData {
        nodes: project_nodes(&data.nodes, to_host_contract_template_node),
    }
}

pub(in super::super) fn to_host_contract_asset_browser_pane(
    data: AssetBrowserPaneViewData,
    pane_size: PaneContentSize,
) -> host_contract::AssetBrowserPaneData {
    host_contract::AssetBrowserPaneData {
        nodes: project_nodes(&data.nodes, |node| {
            apply_table_layout_context_variant(
                to_host_contract_template_node(node),
                pane_size.width,
            )
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::layouts::common::model_rc;
    use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};

    #[test]
    fn asset_browser_table_nodes_receive_narrow_context_variant() {
        let data = AssetBrowserPaneViewData {
            nodes: model_rc(vec![ViewTemplateNodeData {
                control_id: "WorkbenchAssetBrowserTableHeader".into(),
                role: "Table".into(),
                component_role: "table".into(),
                component_variant: "asset-table".into(),
                frame: ViewTemplateFrameData {
                    width: 420.0,
                    height: 28.0,
                    ..ViewTemplateFrameData::default()
                },
                ..ViewTemplateNodeData::default()
            }]),
        };

        let pane = to_host_contract_asset_browser_pane(
            data,
            PaneContentSize {
                width: 420.0,
                height: 260.0,
            },
        );
        let node = pane.nodes.row_data(0).expect("asset browser table node");

        assert!(
            node.component_variant
                .as_str()
                .split_whitespace()
                .any(|token| token == "layoutNarrow")
        );
    }
}

pub(in super::super) fn to_host_contract_project_overview_pane(
    data: ProjectOverviewPaneViewData,
) -> host_contract::ProjectOverviewPaneData {
    host_contract::ProjectOverviewPaneData {
        nodes: project_nodes(&data.nodes, to_host_contract_template_node),
    }
}
