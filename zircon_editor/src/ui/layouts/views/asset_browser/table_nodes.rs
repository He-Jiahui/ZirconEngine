use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::ViewTemplateNodeData;
use crate::ui::retained_host::primitives::{ModelRc, SharedString};
use crate::ui::workbench::asset_content_layout::BROWSER_CONTENT_ITEM_PREFIX;
use crate::ui::workbench::snapshot::{AssetItemSnapshot, AssetViewMode, AssetWorkspaceSnapshot};
use zircon_runtime_interface::resource::ResourceKind;

use super::name_compaction::{compact_file_like_display_name, RuntimeFileNameCompaction};

const ASSET_TABLE_HEADER_CELLS: [&str; 4] = ["Name", "Type", "Size", "Rev"];
const ASSET_TABLE_NAME_MAX_WIDTH: f32 = 150.0;
const ASSET_TABLE_NAME_FONT_SIZE: f32 = 10.0;
const ASSET_TABLE_NAME_MIN_PREFIX_CHARS: usize = 6;
const ASSET_TABLE_NAME_MIN_TAIL_CHARS: usize = 4;
const ASSET_TABLE_NAME_PREFERRED_TAIL_CHARS: usize = 8;

pub(super) fn asset_table_rows(snapshot: &AssetWorkspaceSnapshot) -> Vec<[String; 4]> {
    snapshot
        .visible_assets
        .iter()
        .map(asset_table_row_cells)
        .collect()
}

pub(super) fn sync_asset_table_nodes(
    nodes: &mut Vec<ViewTemplateNodeData>,
    snapshot: &AssetWorkspaceSnapshot,
) {
    let Some(prototype) = nodes
        .iter()
        .find(|node| asset_table_row_index(node.control_id.as_str()) == Some(0))
        .cloned()
    else {
        return;
    };
    let asset_count = if snapshot.view_mode == AssetViewMode::List {
        snapshot.visible_assets.len()
    } else {
        0
    };
    nodes.retain(|node| {
        asset_table_row_index(node.control_id.as_str())
            .map(|index| index < asset_count)
            .unwrap_or(true)
    });

    for index in 0..asset_count {
        if nodes
            .iter()
            .any(|node| asset_table_row_index(node.control_id.as_str()) == Some(index))
        {
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

pub(super) fn mark_asset_table_rows(
    nodes: &mut [ViewTemplateNodeData],
    snapshot: &AssetWorkspaceSnapshot,
) {
    let selected_uuid = snapshot.selected_asset_uuid.as_deref();
    for index in 0..snapshot.visible_assets.len() {
        let control_id = asset_table_row_control_id(index);
        if let Some(node) = nodes.iter_mut().find(|node| node.control_id == control_id) {
            let selected = snapshot
                .visible_assets
                .get(index)
                .map(|asset| asset.selected || selected_uuid == Some(asset.uuid.as_str()))
                .unwrap_or(false);
            node.selected = selected;
            node.focused = false;
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
        let control_id = asset_table_row_control_id(index);
        if let Some(node) = nodes.iter_mut().find(|node| node.control_id == control_id) {
            node.options = shared_string_options(row.iter().cloned().collect());
            node.text = asset_table_row_text(row).into();
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
mod tests {
    use super::*;
    use crate::ui::retained_host::measure_runtime_text_width;

    #[test]
    fn compact_asset_table_name_preserves_readable_file_identity() {
        assert_eq!(
            compact_asset_table_name("workbench_page_chrome.zui", "zui"),
            "workbench_page_chrome.zui"
        );
        let compact =
            compact_asset_table_name("workbench_extension_accessibility_workspace.zui", "zui");
        assert!(compact.starts_with("workbench"));
        assert!(compact.contains("..."));
        assert!(compact.ends_with(".zui"));
        assert!(
            measure_runtime_text_width(&compact, ASSET_TABLE_NAME_FONT_SIZE)
                <= ASSET_TABLE_NAME_MAX_WIDTH + 0.01,
            "compact asset table name should fit measured table name width: {compact}"
        );
    }

    #[test]
    fn compact_asset_table_name_uses_runtime_width_not_character_count() {
        let narrow = format!("{}.zui", "i".repeat(40));
        let wide = format!("{}.zui", "W".repeat(40));
        assert_eq!(narrow.chars().count(), wide.chars().count());

        assert_eq!(compact_asset_table_name(&narrow, "zui"), narrow);
        let compact_wide = compact_asset_table_name(&wide, "zui");

        assert_ne!(compact_wide, wide);
        assert!(compact_wide.ends_with(".zui"));
        assert!(
            measure_runtime_text_width(&compact_wide, ASSET_TABLE_NAME_FONT_SIZE)
                <= ASSET_TABLE_NAME_MAX_WIDTH + 0.01,
            "wide table name should fit measured width: {compact_wide}"
        );
    }

    #[test]
    fn asset_table_rows_follow_the_real_catalog_without_padding_or_truncation() {
        let snapshot = AssetWorkspaceSnapshot {
            visible_assets: (0..7)
                .map(|index| AssetItemSnapshot {
                    uuid: format!("asset-{index}"),
                    locator: format!("res://Asset_{index}.mesh"),
                    display_name: format!("Asset_{index}.mesh"),
                    file_name: format!("Asset_{index}.mesh"),
                    extension: "mesh".to_string(),
                    kind: ResourceKind::Mesh,
                    asset_type: crate::ui::workbench::snapshot::AssetTypeProjectionSnapshot::from_resource_kind(ResourceKind::Mesh),
                    preview_artifact_path: String::new(),
                    dirty: false,
                    diagnostics: Vec::new(),
                    selected: false,
                    resource_state: None,
                    resource_revision: Some(1),
                })
                .collect(),
            ..AssetWorkspaceSnapshot::default()
        };

        let rows = asset_table_rows(&snapshot);

        assert_eq!(rows.len(), 7);
        assert!(rows.iter().all(|row| row[0] != "Empty Asset"));
        assert_eq!(rows[6][0], "Asset_6.mesh");
    }
}
