use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::ViewTemplateNodeData;
use crate::ui::retained_host::primitives::{ModelRc, SharedString};
use crate::ui::workbench::snapshot::{AssetItemSnapshot, AssetWorkspaceSnapshot};
use zircon_runtime_interface::resource::ResourceKind;

use super::compact_resource_kind_label;

const ASSET_TABLE_HEADER_CELLS: [&str; 4] = ["Name", "Type", "Size", "Rev"];

pub(super) fn asset_table_rows(snapshot: &AssetWorkspaceSnapshot) -> Vec<[String; 4]> {
    let mut rows = snapshot
        .visible_assets
        .iter()
        .take(4)
        .map(asset_table_row_cells)
        .collect::<Vec<_>>();
    while rows.len() < 4 {
        rows.push([
            "Empty Asset".to_string(),
            "Asset".to_string(),
            "0KB".to_string(),
            "pending".to_string(),
        ]);
    }
    rows
}

pub(super) fn mark_asset_table_rows(
    nodes: &mut [ViewTemplateNodeData],
    snapshot: &AssetWorkspaceSnapshot,
) {
    let selected_uuid = snapshot.selected_asset_uuid.as_deref();
    for index in 0..4 {
        let control_id = format!("WorkbenchAssetBrowserAssetRow{:02}", index + 1);
        if let Some(node) = nodes.iter_mut().find(|node| node.control_id == control_id) {
            let selected = snapshot
                .visible_assets
                .get(index)
                .map(|asset| asset.selected || selected_uuid == Some(asset.uuid.as_str()))
                .unwrap_or(false);
            node.selected = selected;
            node.focused = selected;
        }
    }
}

pub(super) fn apply_asset_browser_table_cells(
    nodes: &mut [ViewTemplateNodeData],
    rows: &[[String; 4]],
) {
    if let Some(header) = nodes
        .iter_mut()
        .find(|node| node.control_id == "WorkbenchAssetBrowserTableHeader")
    {
        header.options = shared_string_options(
            ASSET_TABLE_HEADER_CELLS
                .iter()
                .map(|cell| (*cell).to_string())
                .collect(),
        );
        header.text = ASSET_TABLE_HEADER_CELLS.join(" ").into();
    }

    for (index, row) in rows.iter().enumerate() {
        let control_id = format!("WorkbenchAssetBrowserAssetRow{:02}", index + 1);
        if let Some(node) = nodes.iter_mut().find(|node| node.control_id == control_id) {
            node.options = shared_string_options(row.iter().cloned().collect());
            node.text = asset_table_row_text(row).into();
        }
    }
}

fn asset_table_row_cells(asset: &AssetItemSnapshot) -> [String; 4] {
    [
        compact_asset_table_name(&asset.display_name).to_string(),
        compact_resource_kind_label(asset.kind).to_string(),
        asset_size_hint(asset).to_string(),
        asset
            .resource_revision
            .map(|revision| format!("r{revision}"))
            .unwrap_or_else(|| "new".to_string()),
    ]
}

pub(super) fn asset_table_row_text(row: &[String; 4]) -> String {
    row.join(" ")
}

fn asset_size_hint(asset: &AssetItemSnapshot) -> &'static str {
    match asset.kind {
        ResourceKind::Texture => "1.2M",
        ResourceKind::Material | ResourceKind::MaterialGraph | ResourceKind::Shader => "512K",
        ResourceKind::Scene | ResourceKind::Prefab | ResourceKind::UiLayout => "64K",
        ResourceKind::Model | ResourceKind::Mesh | ResourceKind::AnimationClip => "2.4M",
        _ => "16K",
    }
}

fn compact_asset_table_name(display_name: &str) -> &'static str {
    let lower = display_name.to_ascii_lowercase();
    if lower.contains("workbench_host") {
        "Host"
    } else if lower.contains("editor_base") {
        "Base"
    } else if lower.contains("folder") {
        "Folder"
    } else if lower.contains("accessibility") {
        "A11y"
    } else if lower.contains("material") {
        "Mat"
    } else if lower.contains("scene") {
        "Scene"
    } else {
        "Asset"
    }
}

fn shared_string_options(values: Vec<String>) -> ModelRc<SharedString> {
    model_rc(values.into_iter().map(SharedString::from).collect())
}
