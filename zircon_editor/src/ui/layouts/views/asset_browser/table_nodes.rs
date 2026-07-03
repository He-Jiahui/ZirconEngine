use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::ViewTemplateNodeData;
use crate::ui::retained_host::primitives::{ModelRc, SharedString};
use crate::ui::workbench::snapshot::{AssetItemSnapshot, AssetWorkspaceSnapshot};
use zircon_runtime_interface::resource::ResourceKind;

use super::labels::compact_resource_kind_label;

const ASSET_TABLE_HEADER_CELLS: [&str; 4] = ["Name", "Type", "Size", "Rev"];
const ASSET_TABLE_NAME_CHAR_BUDGET: usize = 32;
const ASSET_TABLE_NAME_PREFIX_CHARS: usize = 18;
const ASSET_TABLE_NAME_TAIL_CHARS: usize = 8;
const ASSET_TABLE_NAME_ELLIPSIS: &str = "...";

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
    let name = display_name.trim();
    if name.chars().count() <= ASSET_TABLE_NAME_CHAR_BUDGET {
        return name.to_string();
    }

    if let Some(file_name) = extension_preserving_table_name(name, extension) {
        return file_name;
    }

    let prefix = first_n_chars(name, ASSET_TABLE_NAME_PREFIX_CHARS);
    let tail = last_n_chars(name, ASSET_TABLE_NAME_TAIL_CHARS);
    format!("{prefix}{ASSET_TABLE_NAME_ELLIPSIS}{tail}")
}

fn extension_preserving_table_name(display_name: &str, extension: &str) -> Option<String> {
    let extension = extension.trim().trim_start_matches('.');
    let (stem, suffix) = display_name.rsplit_once('.')?;
    if extension.is_empty() || !suffix.eq_ignore_ascii_case(extension) {
        return None;
    }

    let suffix_chars = suffix.chars().count();
    let ellipsis_chars = ASSET_TABLE_NAME_ELLIPSIS.chars().count();
    let stem_budget =
        ASSET_TABLE_NAME_CHAR_BUDGET.saturating_sub(ellipsis_chars + 1 + suffix_chars);
    if stem_budget < 8 {
        return None;
    }

    let tail_chars = ASSET_TABLE_NAME_TAIL_CHARS.min(stem_budget / 2);
    let prefix_chars = ASSET_TABLE_NAME_PREFIX_CHARS.min(stem_budget.saturating_sub(tail_chars));
    if stem.chars().count() <= prefix_chars + tail_chars {
        return Some(display_name.to_string());
    }

    let prefix = first_n_chars(stem, prefix_chars);
    let tail = last_n_chars(stem, tail_chars);
    Some(format!(
        "{prefix}{ASSET_TABLE_NAME_ELLIPSIS}{tail}.{suffix}"
    ))
}

fn first_n_chars(text: &str, count: usize) -> String {
    text.chars().take(count).collect()
}

fn last_n_chars(text: &str, count: usize) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    let start = chars.len().saturating_sub(count);
    chars[start..].iter().copied().collect()
}

fn shared_string_options(values: Vec<String>) -> ModelRc<SharedString> {
    model_rc(values.into_iter().map(SharedString::from).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_asset_table_name_preserves_readable_file_identity() {
        assert_eq!(
            compact_asset_table_name("workbench_page_chrome.zui", "zui"),
            "workbench_page_chrome.zui"
        );
        assert_eq!(
            compact_asset_table_name("workbench_extension_accessibility_workspace.zui", "zui"),
            "workbench_extensi...orkspace.zui"
        );
    }
}
