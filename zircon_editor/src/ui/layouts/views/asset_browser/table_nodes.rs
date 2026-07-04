use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::ViewTemplateNodeData;
use crate::ui::retained_host::primitives::{ModelRc, SharedString};
use crate::ui::workbench::snapshot::{AssetItemSnapshot, AssetWorkspaceSnapshot};
use zircon_runtime_interface::resource::ResourceKind;

use super::labels::compact_resource_kind_label;
use super::name_compaction::{compact_file_like_display_name, RuntimeFileNameCompaction};

const ASSET_TABLE_HEADER_CELLS: [&str; 4] = ["Name", "Type", "Size", "Rev"];
const ASSET_TABLE_NAME_MAX_WIDTH: f32 = 150.0;
const ASSET_TABLE_NAME_FONT_SIZE: f32 = 10.0;
const ASSET_TABLE_NAME_MIN_PREFIX_CHARS: usize = 6;
const ASSET_TABLE_NAME_MIN_TAIL_CHARS: usize = 4;
const ASSET_TABLE_NAME_PREFERRED_TAIL_CHARS: usize = 8;

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
        let control_id = format!("WorkbenchAssetBrowserAssetRow{:02}", index + 1);
        if let Some(node) = nodes.iter_mut().find(|node| node.control_id == control_id) {
            node.options = shared_string_options(row.iter().cloned().collect());
            node.text = asset_table_row_text(row).into();
        }
    }
}

fn asset_table_row_cells(asset: &AssetItemSnapshot) -> [String; 4] {
    [
        compact_asset_table_name(&asset.display_name, &asset.extension),
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
}
