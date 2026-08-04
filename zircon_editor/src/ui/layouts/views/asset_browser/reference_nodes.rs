use crate::ui::layouts::views::ViewTemplateNodeData;
use crate::ui::workbench::snapshot::AssetWorkspaceSnapshot;

use super::super::asset_reference_rows::{
    AssetReferenceListControls, apply_asset_reference_lists_layout, sync_asset_reference_lists,
};

const LEFT_REFERENCES: AssetReferenceListControls = AssetReferenceListControls {
    title_control_id: "AssetBrowserReferenceLeftTitleText",
    empty_control_id: "AssetBrowserReferenceLeftEmptyText",
    panel_control_id: "AssetBrowserReferenceLeftPanel",
    scroll_body_control_id: "AssetBrowserReferenceLeftScrollBody",
    row_panel_control_id: "AssetBrowserReferenceLeftRowPanel",
    row_name_control_id: "AssetBrowserReferenceLeftRowNameText",
    row_locator_control_id: "AssetBrowserReferenceLeftRowLocatorText",
    row_kind_control_id: "AssetBrowserReferenceLeftRowKindText",
    node_id_scope: "asset_browser.references.left",
    title: "References",
    empty_text: "No direct references",
};

const RIGHT_USED_BY: AssetReferenceListControls = AssetReferenceListControls {
    title_control_id: "AssetBrowserReferenceRightTitleText",
    empty_control_id: "AssetBrowserReferenceRightEmptyText",
    panel_control_id: "AssetBrowserReferenceRightPanel",
    scroll_body_control_id: "AssetBrowserReferenceRightScrollBody",
    row_panel_control_id: "AssetBrowserReferenceRightRowPanel",
    row_name_control_id: "AssetBrowserReferenceRightRowNameText",
    row_locator_control_id: "AssetBrowserReferenceRightRowLocatorText",
    row_kind_control_id: "AssetBrowserReferenceRightRowKindText",
    node_id_scope: "asset_browser.references.right",
    title: "Used By",
    empty_text: "No usages",
};

pub(super) fn sync_asset_browser_reference_nodes(
    nodes: &mut Vec<ViewTemplateNodeData>,
    snapshot: &AssetWorkspaceSnapshot,
) {
    sync_asset_reference_lists(nodes, snapshot, LEFT_REFERENCES, RIGHT_USED_BY);
}

pub(super) fn apply_asset_browser_reference_layout(nodes: &mut [ViewTemplateNodeData]) {
    apply_asset_reference_lists_layout(
        nodes,
        "AssetBrowserUtilityContentPanel",
        LEFT_REFERENCES,
        RIGHT_USED_BY,
    );
}
