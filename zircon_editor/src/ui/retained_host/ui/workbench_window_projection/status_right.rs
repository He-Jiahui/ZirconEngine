use crate::ui::retained_host as host_contract;
use crate::ui::template_runtime::RetainedUiHostNodeModel;

use super::node_index::ProjectionNodeIndex;
use super::properties::{color_property, numeric_property};

const WORKBENCH_STATUS_RIGHT_OFFSET_Y: f64 = -0.5;
const WORKBENCH_STATUS_RIGHT_LABEL_COLOR: host_contract::primitives::Color =
    host_contract::primitives::Color::from_rgb_u8(125, 137, 144);

pub(super) fn inherited_status_right_numeric_property(
    node: &RetainedUiHostNodeModel,
    node_index: &ProjectionNodeIndex<'_>,
    property: &str,
) -> Option<f64> {
    inherited_status_right_parent(node, node_index)
        .and_then(|parent| numeric_property(&parent.properties, property))
        .or_else(|| {
            (property == "status_right_offset_y" && is_status_right_control(node))
                .then_some(WORKBENCH_STATUS_RIGHT_OFFSET_Y)
        })
}

pub(super) fn inherited_status_right_color_property(
    node: &RetainedUiHostNodeModel,
    node_index: &ProjectionNodeIndex<'_>,
    property: &str,
) -> Option<host_contract::primitives::Color> {
    inherited_status_right_parent(node, node_index)
        .and_then(|parent| color_property(&parent.properties, property))
        .or_else(|| {
            (property == "status_right_label_color" && is_status_right_control(node))
                .then_some(WORKBENCH_STATUS_RIGHT_LABEL_COLOR)
        })
}

fn inherited_status_right_parent<'a>(
    node: &RetainedUiHostNodeModel,
    node_index: &ProjectionNodeIndex<'a>,
) -> Option<&'a RetainedUiHostNodeModel> {
    if !is_status_right_control(node) {
        return None;
    }

    let mut parent_id = node.parent_id.as_deref();
    for _ in 0..node_index.node_count() {
        let current_parent_id = parent_id?;
        let parent = node_index.node(current_parent_id)?;
        if parent.control_id.as_deref() == Some("WorkbenchWindowStatusBar") {
            return Some(parent);
        }
        parent_id = parent.parent_id.as_deref();
    }
    None
}

fn is_status_right_control(node: &RetainedUiHostNodeModel) -> bool {
    matches!(
        node.control_id.as_deref(),
        Some(
            "WorkbenchStatusGrid"
                | "WorkbenchStatusSnap"
                | "WorkbenchStatusTaskProgress"
                | "WorkbenchStatusTaskLabel"
                | "WorkbenchStatusTaskBar"
                | "WorkbenchStatusSnapToggle"
                | "WorkbenchStatusWorld"
                | "WorkbenchStatusTarget"
                | "WorkbenchStatusZoom"
        )
    )
}
