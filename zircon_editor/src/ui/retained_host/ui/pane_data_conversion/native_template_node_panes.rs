use crate::ui::layouts::windows::workbench_host_window::{
    AssetBrowserPaneViewData, AssetsActivityPaneViewData, ProjectOverviewPaneViewData,
};
use crate::ui::retained_host as host_contract;

use super::super::template_node_conversion::to_host_contract_template_node_owned;
use super::template_node_projection::project_nodes;

pub(in super::super) fn to_host_contract_assets_activity_pane(
    data: AssetsActivityPaneViewData,
) -> host_contract::AssetsActivityPaneData {
    host_contract::AssetsActivityPaneData {
        nodes: project_nodes(&data.nodes, to_host_contract_template_node_owned),
    }
}

pub(in super::super) fn to_host_contract_asset_browser_pane(
    data: AssetBrowserPaneViewData,
) -> host_contract::AssetBrowserPaneData {
    host_contract::AssetBrowserPaneData {
        nodes: project_nodes(&data.nodes, to_host_contract_template_node_owned),
    }
}

pub(in super::super) fn to_host_contract_project_overview_pane(
    data: ProjectOverviewPaneViewData,
) -> host_contract::ProjectOverviewPaneData {
    host_contract::ProjectOverviewPaneData {
        nodes: project_nodes(&data.nodes, to_host_contract_template_node_owned),
    }
}
