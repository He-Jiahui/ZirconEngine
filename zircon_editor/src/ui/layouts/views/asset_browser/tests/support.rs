use super::*;

pub(super) fn asset_item(index: usize, selected: bool) -> AssetItemSnapshot {
    AssetItemSnapshot {
        uuid: format!("asset-{index:02}"),
        locator: format!("res://asset-{index:02}"),
        display_name: format!("Asset_{index:02}.mesh"),
        file_name: format!("Asset_{index:02}.mesh"),
        extension: "mesh".to_string(),
        kind: ResourceKind::Mesh,
        asset_type: crate::ui::workbench::snapshot::AssetTypeProjectionSnapshot::from_resource_kind(
            ResourceKind::Mesh,
        ),
        preview_artifact_path: String::new(),
        dirty: false,
        diagnostics: Vec::new(),
        selected,
        resource_state: None,
        resource_revision: Some(index as u64),
    }
}

pub(super) fn asset_folder(
    folder_id: &str,
    display_name: &str,
    depth: usize,
    selected: bool,
) -> AssetFolderSnapshot {
    AssetFolderSnapshot {
        folder_id: folder_id.to_string(),
        parent_folder_id: None,
        display_name: display_name.to_string(),
        recursive_asset_count: 0,
        depth,
        selected,
    }
}

pub(super) fn find_node(
    nodes: &crate::ui::retained_host::primitives::ModelRc<ViewTemplateNodeData>,
    control_id: &str,
) -> ViewTemplateNodeData {
    for index in 0..nodes.row_count() {
        let Some(node) = nodes.row_data(index) else {
            continue;
        };
        if node.control_id.as_str() == control_id {
            return node;
        }
    }
    panic!("missing node {control_id}");
}

pub(super) fn assert_control_absent(
    nodes: &crate::ui::retained_host::primitives::ModelRc<ViewTemplateNodeData>,
    control_id: &str,
) {
    for index in 0..nodes.row_count() {
        let Some(node) = nodes.row_data(index) else {
            continue;
        };
        assert_ne!(
            node.control_id.as_str(),
            control_id,
            "thumbnail mode should not project `{control_id}`"
        );
    }
}
