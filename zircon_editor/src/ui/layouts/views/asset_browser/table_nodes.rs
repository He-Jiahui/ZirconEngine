use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::ViewTemplateNodeData;
use crate::ui::retained_host::primitives::{ModelRc, SharedString};
use crate::ui::workbench::asset_content_layout::BROWSER_CONTENT_ITEM_PREFIX;
use crate::ui::workbench::asset_content_layout::{
    AssetBrowserListPaintItem, AssetBrowserLogicalPaintGeneration, AssetBrowserPaintItem,
    BROWSER_CONTENT_LIST_ROW_HEIGHT, BROWSER_CONTENT_TABLE_CONTROL_ID,
};
use crate::ui::workbench::snapshot::{AssetItemSnapshot, AssetViewMode, AssetWorkspaceSnapshot};
use zircon_runtime_interface::resource::ResourceKind;
use zircon_runtime_interface::ui::design_tokens::EditorTypographyTokens;

use super::name_compaction::{compact_file_like_display_name, RuntimeFileNameCompaction};

const ASSET_TABLE_HEADER_CELLS: [&str; 4] = ["Name", "Type", "Size", "Rev"];
const ASSET_TABLE_NAME_MAX_WIDTH: f32 = 150.0;
const ASSET_TABLE_NAME_FONT_SIZE: f32 = EditorTypographyTokens::WORKBENCH_CAPTION_SIZE;
const ASSET_TABLE_NAME_MIN_PREFIX_CHARS: usize = 6;
const ASSET_TABLE_NAME_MIN_TAIL_CHARS: usize = 4;
const ASSET_TABLE_NAME_PREFERRED_TAIL_CHARS: usize = 8;

pub(super) fn sync_asset_table_nodes(
    nodes: &mut Vec<ViewTemplateNodeData>,
    view_mode: AssetViewMode,
    materialized_item_count: usize,
) {
    let Some(prototype) = nodes
        .iter()
        .find(|node| asset_table_row_index(node.control_id.as_str()) == Some(0))
        .cloned()
    else {
        return;
    };
    let asset_count = if view_mode == AssetViewMode::List {
        materialized_item_count
    } else {
        0
    };
    let mut existing_row_indices = vec![false; asset_count];
    nodes.retain(
        |node| match asset_table_row_index(node.control_id.as_str()) {
            Some(index) if index < asset_count => {
                existing_row_indices[index] = true;
                true
            }
            Some(_) => false,
            None => true,
        },
    );
    let missing_row_count = existing_row_indices
        .iter()
        .filter(|exists| !**exists)
        .count();
    nodes.reserve(missing_row_count);

    for index in 0..asset_count {
        if existing_row_indices[index] {
            continue;
        }
        let mut row = prototype.clone();
        row.node_id = format!("asset_browser.runtime.asset_row_{index:02}").into();
        row.control_id = asset_table_row_control_id(index).into();
        row.selected = false;
        row.focused = false;
        row.hovered = false;
        nodes.push(row);
    }
}

pub(super) fn asset_browser_list_paint_item(asset: &AssetItemSnapshot) -> AssetBrowserPaintItem {
    let cells = asset_table_row_cells(asset);
    let text = asset_table_row_text(&cells);
    AssetBrowserPaintItem::List(AssetBrowserListPaintItem {
        text,
        cells: shared_string_options(cells.into_iter().collect()),
    })
}

pub(super) fn apply_asset_browser_list_logical_extent(
    nodes: &mut [ViewTemplateNodeData],
    logical_item_count: usize,
) {
    if let Some(table) = nodes
        .iter_mut()
        .find(|node| node.control_id == BROWSER_CONTENT_TABLE_CONTROL_ID)
    {
        table.value_number = BROWSER_CONTENT_LIST_ROW_HEIGHT * logical_item_count as f32;
    }
}

pub(super) fn mark_asset_table_rows(
    nodes: &mut [ViewTemplateNodeData],
    snapshot: &AssetWorkspaceSnapshot,
) {
    let selected_uuid = snapshot.selected_asset_uuid.as_deref();
    for node in nodes.iter_mut() {
        let Some(index) = asset_table_row_index(node.control_id.as_str()) else {
            continue;
        };
        let selected = snapshot
            .visible_assets
            .get(index)
            .map(|asset| asset.selected || selected_uuid == Some(asset.uuid.as_str()))
            .unwrap_or(false);
        node.selected = selected;
        node.focused = false;
    }
}

pub(super) fn apply_asset_browser_table_cells(
    nodes: &mut [ViewTemplateNodeData],
    items: &AssetBrowserLogicalPaintGeneration,
) {
    for node in nodes.iter_mut() {
        if node.control_id == "WorkbenchAssetBrowserTableHeader" {
            node.options = shared_string_options(
                ASSET_TABLE_HEADER_CELLS
                    .iter()
                    .map(|cell| (*cell).to_string())
                    .collect(),
            );
            node.text = ASSET_TABLE_HEADER_CELLS.join(" ").into();
        } else if let Some(AssetBrowserPaintItem::List(item)) =
            asset_table_row_index(node.control_id.as_str()).and_then(|index| items.get(index))
        {
            node.options = item.cells.clone();
            node.text = item.text.clone().into();
        }
    }
}

pub(super) fn asset_table_row_control_id(index: usize) -> String {
    format!("{BROWSER_CONTENT_ITEM_PREFIX}{:02}", index + 1)
}

pub(super) fn asset_table_row_index(control_id: &str) -> Option<usize> {
    control_id
        .strip_prefix(BROWSER_CONTENT_ITEM_PREFIX)?
        .parse::<usize>()
        .ok()?
        .checked_sub(1)
}

fn asset_table_row_cells(asset: &AssetItemSnapshot) -> [String; 4] {
    [
        compact_asset_table_name(&asset.display_name, &asset.extension),
        asset.asset_type.display_name.clone(),
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

fn compact_asset_table_name(display_name: &str, extension: &str) -> String {
    compact_file_like_display_name(
        display_name,
        extension,
        RuntimeFileNameCompaction {
            max_width: ASSET_TABLE_NAME_MAX_WIDTH,
            font_size: ASSET_TABLE_NAME_FONT_SIZE,
            min_prefix_chars: ASSET_TABLE_NAME_MIN_PREFIX_CHARS,
            min_tail_stem_chars: ASSET_TABLE_NAME_MIN_TAIL_CHARS,
            preferred_tail_stem_chars: ASSET_TABLE_NAME_PREFERRED_TAIL_CHARS,
        },
    )
}

fn shared_string_options(values: Vec<String>) -> ModelRc<SharedString> {
    model_rc(values.into_iter().map(SharedString::from).collect())
}

#[cfg(test)]
fn asset_table_rows(snapshot: &AssetWorkspaceSnapshot) -> Vec<[String; 4]> {
    snapshot
        .visible_assets
        .iter()
        .map(asset_table_row_cells)
        .collect()
}

#[cfg(test)]
#[path = "tests/table_nodes.rs"]
mod tests;
