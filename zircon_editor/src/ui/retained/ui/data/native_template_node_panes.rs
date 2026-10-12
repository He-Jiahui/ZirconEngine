//! 这些原生 Pane 通过统一节点转换进入宿主；资产浏览器额外投影窄表格布局，保留运行时绘制帧的所有权。
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
        render_source_frame: data.render_source_frame,
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
        render_source_frame: data.render_source_frame,
    }
}

#[cfg(test)]
#[path = "tests/native_template_node_panes.rs"]
mod tests;

pub(in super::super) fn to_host_contract_project_overview_pane(
    data: ProjectOverviewPaneViewData,
) -> host_contract::ProjectOverviewPaneData {
    host_contract::ProjectOverviewPaneData {
        nodes: project_nodes(&data.nodes, to_host_contract_template_node),
    }
}
