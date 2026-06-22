use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::windows::workbench_host_window::{PaneContentSize, PaneData};
use crate::ui::retained_host as host_contract;

use super::super::template_node_conversion::to_host_contract_template_node_owned;
use super::template_node_projection::project_nodes;

pub(crate) fn to_host_contract_generated_bottom_pane_from_host_pane(
    data: &PaneData,
    content_size: PaneContentSize,
) -> host_contract::GeneratedBottomPaneData {
    let nodes = data
        .pane_presentation
        .as_ref()
        .and_then(|presentation| {
            super::project_pane_template_nodes(&presentation.body, content_size)
        })
        .map(model_rc)
        .unwrap_or_else(|| {
            project_nodes(
                &data.native_body.generated_bottom.nodes,
                to_host_contract_template_node_owned,
            )
        });

    host_contract::GeneratedBottomPaneData {
        nodes,
        status: data.native_body.generated_bottom.status.clone(),
    }
}
