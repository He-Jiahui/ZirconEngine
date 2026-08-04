use crate::ui::layouts::views::ViewTemplateNodeData;
use crate::ui::workbench::snapshot::AssetWorkspaceSnapshot;

use super::super::asset_reference_rows::{
    AssetReferenceListControls, apply_asset_reference_lists_layout, sync_asset_reference_lists,
};

const LEFT_REFERENCES: AssetReferenceListControls = AssetReferenceListControls {
    title_control_id: "AssetsActivityReferenceLeftTitleText",
    empty_control_id: "AssetsActivityReferenceLeftEmptyText",
    panel_control_id: "AssetsActivityReferenceLeftPanel",
    scroll_body_control_id: "AssetsActivityReferenceLeftScrollBody",
    row_panel_control_id: "AssetsActivityReferenceLeftRowPanel",
    row_name_control_id: "AssetsActivityReferenceLeftRowNameText",
    row_locator_control_id: "AssetsActivityReferenceLeftRowLocatorText",
    row_kind_control_id: "AssetsActivityReferenceLeftRowKindText",
    node_id_scope: "assets_activity.references.left",
    title: "References",
    empty_text: "No direct references",
};

const RIGHT_USED_BY: AssetReferenceListControls = AssetReferenceListControls {
    title_control_id: "AssetsActivityReferenceRightTitleText",
    empty_control_id: "AssetsActivityReferenceRightEmptyText",
    panel_control_id: "AssetsActivityReferenceRightPanel",
    scroll_body_control_id: "AssetsActivityReferenceRightScrollBody",
    row_panel_control_id: "AssetsActivityReferenceRightRowPanel",
    row_name_control_id: "AssetsActivityReferenceRightRowNameText",
    row_locator_control_id: "AssetsActivityReferenceRightRowLocatorText",
    row_kind_control_id: "AssetsActivityReferenceRightRowKindText",
    node_id_scope: "assets_activity.references.right",
    title: "Used By",
    empty_text: "No usages",
};

pub(super) fn sync_assets_activity_reference_nodes(
    nodes: &mut Vec<ViewTemplateNodeData>,
    snapshot: &AssetWorkspaceSnapshot,
) {
    sync_asset_reference_lists(nodes, snapshot, LEFT_REFERENCES, RIGHT_USED_BY);
}

pub(super) fn apply_assets_activity_reference_layout(nodes: &mut [ViewTemplateNodeData]) {
    apply_asset_reference_lists_layout(
        nodes,
        "AssetsActivityUtilityContentPanel",
        LEFT_REFERENCES,
        RIGHT_USED_BY,
    );
}
