use crate::ui::layouts::views::ViewTemplateNodeData;
use crate::ui::workbench::snapshot::AssetWorkspaceSnapshot;

const SOURCE_ROW_PANEL_CONTROL_ID: &str = "AssetBrowserSourcesRowPanel";
const SOURCE_TREE_ROW_PREFIX: &str = "AssetBrowserSourcesTreeRow";
const SOURCE_ROW_CHILD_CONTROL_IDS: [&str; 3] = [
    "AssetBrowserSourcesRowIcon",
    "AssetBrowserSourcesRowNameText",
    "AssetBrowserSourcesRowCountText",
];

pub(super) fn sync_asset_browser_source_tree_nodes(
    nodes: &mut Vec<ViewTemplateNodeData>,
    snapshot: &AssetWorkspaceSnapshot,
) {
    let Some(prototype) = nodes
        .iter()
        .find(|node| node.control_id == SOURCE_ROW_PANEL_CONTROL_ID)
        .cloned()
    else {
        return;
    };

    nodes.retain(|node| {
        !is_source_tree_row(node.control_id.as_str())
            && !SOURCE_ROW_CHILD_CONTROL_IDS.contains(&node.control_id.as_str())
    });

    if snapshot.folder_tree.is_empty() {
        nodes.push(source_tree_row_node(
            &prototype,
            0,
            source_tree_fallback_label(snapshot),
            snapshot.visible_assets.len(),
            snapshot.selected_folder_id.is_some(),
            0,
        ));
        return;
    }

    for (index, folder) in snapshot.folder_tree.iter().enumerate() {
        nodes.push(source_tree_row_node(
            &prototype,
            index,
            folder.display_name.clone(),
            folder.recursive_asset_count,
            folder.selected
                || snapshot.selected_folder_id.as_deref() == Some(folder.folder_id.as_str()),
            folder.depth,
        ));
    }
}

fn source_tree_row_node(
    prototype: &ViewTemplateNodeData,
    index: usize,
    text: String,
    recursive_asset_count: usize,
    selected: bool,
    depth: usize,
) -> ViewTemplateNodeData {
    let mut row = prototype.clone();
    row.node_id = format!("asset_browser.source_tree.row_{:02}", index + 1).into();
    row.control_id = source_tree_row_control_id(index).into();
    row.role = "TreeRow".into();
    row.component_role = "workbench-tree-row".into();
    row.component_variant = "asset-folder".into();
    row.surface_variant = "asset-tree-row".into();
    row.text = text.into();
    row.value_text = recursive_asset_count.to_string().into();
    row.selected = selected;
    row.focused = false;
    row.hovered = false;
    row.pressed = false;
    row.value_number = depth.min(i32::MAX as usize) as f32;
    row
}

fn source_tree_fallback_label(snapshot: &AssetWorkspaceSnapshot) -> String {
    snapshot
        .visible_folders
        .iter()
        .find(|folder| {
            folder.selected
                || snapshot.selected_folder_id.as_deref() == Some(folder.folder_id.as_str())
        })
        .map(|folder| folder.display_name.clone())
        .filter(|label| !label.trim().is_empty())
        .or_else(|| {
            (!snapshot.project_root.trim().is_empty()).then(|| snapshot.project_root.clone())
        })
        .unwrap_or_else(|| "Content".to_string())
}

pub(super) fn is_source_tree_row(control_id: &str) -> bool {
    if control_id == SOURCE_ROW_PANEL_CONTROL_ID {
        return true;
    }
    let Some(row_number) = control_id
        .strip_prefix(SOURCE_TREE_ROW_PREFIX)
        .and_then(|suffix| suffix.strip_suffix("/AssetBrowserSourcesRowPanel"))
    else {
        return false;
    };
    !row_number.is_empty() && row_number.bytes().all(|byte| byte.is_ascii_digit())
}

pub(super) fn source_tree_row_control_id(index: usize) -> String {
    if index == 0 {
        SOURCE_ROW_PANEL_CONTROL_ID.to_string()
    } else {
        format!(
            "{SOURCE_TREE_ROW_PREFIX}{:02}/{SOURCE_ROW_PANEL_CONTROL_ID}",
            index + 1
        )
    }
}

#[cfg(test)]
mod tests {
    use super::sync_asset_browser_source_tree_nodes;
    use crate::ui::layouts::views::ViewTemplateNodeData;
    use crate::ui::workbench::snapshot::{AssetFolderSnapshot, AssetWorkspaceSnapshot};

    #[test]
    fn source_tree_nodes_preserve_pointer_order_selection_and_depth() {
        let snapshot = AssetWorkspaceSnapshot {
            selected_folder_id: Some("materials".to_string()),
            folder_tree: vec![
                folder("content", None, "Content", 0, false),
                folder("materials", Some("content"), "Materials", 1, false),
                folder("textures", Some("content"), "Textures", 1, false),
            ],
            ..AssetWorkspaceSnapshot::default()
        };
        let mut nodes = vec![source_row_prototype()];

        sync_asset_browser_source_tree_nodes(&mut nodes, &snapshot);

        let rows = nodes
            .iter()
            .filter(|node| node.role == "TreeRow")
            .collect::<Vec<_>>();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].control_id, "AssetBrowserSourcesRowPanel");
        assert_eq!(
            rows[1].control_id,
            "AssetBrowserSourcesTreeRow02/AssetBrowserSourcesRowPanel"
        );
        assert_eq!(rows[0].text, "Content");
        assert_eq!(rows[1].text, "Materials");
        assert_eq!(rows[2].text, "Textures");
        assert_eq!(rows[1].value_number, 1.0);
        assert!(rows[1].selected);
        assert!(!rows[0].selected);
    }

    fn source_row_prototype() -> ViewTemplateNodeData {
        ViewTemplateNodeData {
            node_id: "asset_browser.sources.row".into(),
            control_id: "AssetBrowserSourcesRowPanel".into(),
            role: "Panel".into(),
            ..ViewTemplateNodeData::default()
        }
    }

    fn folder(
        folder_id: &str,
        parent_folder_id: Option<&str>,
        display_name: &str,
        depth: usize,
        selected: bool,
    ) -> AssetFolderSnapshot {
        AssetFolderSnapshot {
            folder_id: folder_id.to_string(),
            parent_folder_id: parent_folder_id.map(str::to_string),
            display_name: display_name.to_string(),
            recursive_asset_count: 0,
            depth,
            selected,
        }
    }
}
