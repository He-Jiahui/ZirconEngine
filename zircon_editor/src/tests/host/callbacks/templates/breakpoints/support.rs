use super::super::super::support::{BuiltinWorkbenchWindowTemplateSurfaceBridge, UiFrame};
use crate::ui::retained_host::{to_host_contract_workbench_window_nodes, TemplatePaneNodeData};

pub(super) const COMPACT_WORKBENCH_WIDTH: u32 = 900;
pub(super) const COMPACT_WORKBENCH_HEIGHT: u32 = 620;
pub(super) const NARROW_WORKBENCH_WIDTH: u32 = 640;
pub(super) const NARROW_WORKBENCH_HEIGHT: u32 = 520;
pub(super) const FULL_WORKBENCH_WIDTH: u32 = 1672;
pub(super) const FULL_WORKBENCH_HEIGHT: u32 = 941;

pub(super) fn workbench_window_node(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
) -> TemplatePaneNodeData {
    let nodes = to_host_contract_workbench_window_nodes(Some(bridge.host_projection()));
    (0..nodes.row_count())
        .filter_map(|row| nodes.row_data(row))
        .find(|node| node.control_id.as_str() == control_id)
        .unwrap_or_else(|| panic!("{control_id} should project to native host nodes"))
}

pub(super) fn assert_frame_value(label: &str, actual: f32, expected: f32) {
    const EPSILON: f32 = 0.01;
    assert!(
        (actual - expected).abs() <= EPSILON,
        "{label} should be {expected}, got {actual}"
    );
}

pub(super) fn rendered_control_frame(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
) -> UiFrame {
    let node_id = bridge
        .surface()
        .tree
        .nodes
        .values()
        .find_map(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                .filter(|candidate| *candidate == control_id)
                .map(|_| node.node_id)
        })
        .unwrap_or_else(|| panic!("{control_id} should resolve to one runtime node"));
    bridge
        .surface()
        .render_extract
        .list
        .commands
        .iter()
        .filter(|command| command.node_id == node_id)
        .map(|command| command.frame)
        .max_by(|left, right| {
            let left_area = left.width.max(0.0) * left.height.max(0.0);
            let right_area = right.width.max(0.0) * right.height.max(0.0);
            left_area.total_cmp(&right_area)
        })
        .unwrap_or_else(|| panic!("{control_id} should emit popup render commands"))
}
