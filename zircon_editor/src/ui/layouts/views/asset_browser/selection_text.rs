use std::collections::{HashMap, HashSet};

use crate::ui::workbench::snapshot::{
    AssetFolderSnapshot, AssetItemSnapshot, AssetSelectionSnapshot, AssetWorkspaceSnapshot,
};

pub(super) fn selected_folder(snapshot: &AssetWorkspaceSnapshot) -> Option<&AssetFolderSnapshot> {
    let selected_folder_id = snapshot.selected_folder_id.as_deref()?;
    snapshot
        .visible_folders
        .iter()
        .chain(snapshot.folder_tree.iter())
        .find(|folder| folder.folder_id == selected_folder_id)
}

pub(super) fn selected_folder_breadcrumb(snapshot: &AssetWorkspaceSnapshot) -> Option<String> {
    let selected = selected_folder(snapshot)?;
    let mut folders = HashMap::<&str, &AssetFolderSnapshot>::with_capacity(
        snapshot.folder_tree.len() + snapshot.visible_folders.len(),
    );
    for folder in snapshot
        .folder_tree
        .iter()
        .chain(snapshot.visible_folders.iter())
    {
        folders.entry(folder.folder_id.as_str()).or_insert(folder);
    }

    let mut segments: Vec<&str> = vec![selected.display_name.as_str()];
    let mut visited = HashSet::<&str>::new();
    visited.insert(selected.folder_id.as_str());
    let mut parent_id = selected.parent_folder_id.as_deref();

    while let Some(id) = parent_id {
        if !visited.insert(id) {
            break;
        }
        let Some(parent) = folders.get(id).copied() else {
            break;
        };
        segments.push(parent.display_name.as_str());
        parent_id = parent.parent_folder_id.as_deref();
    }
    segments.reverse();
    Some(format!(
        "{} / {} assets",
        segments.join(" / "),
        selected.recursive_asset_count
    ))
}

pub(super) fn selected_asset(snapshot: &AssetWorkspaceSnapshot) -> Option<&AssetItemSnapshot> {
    let selected_uuid_index = snapshot
        .selected_asset_uuid
        .as_deref()
        .and_then(|uuid| snapshot.visible_assets.selected_index(uuid));
    let selected_flag_index = snapshot.visible_assets.selected_indices().first().copied();
    let selected_index = match (selected_uuid_index, selected_flag_index) {
        (Some(uuid_index), Some(flag_index)) => Some(uuid_index.min(flag_index)),
        (Some(index), None) | (None, Some(index)) => Some(index),
        (None, None) => None,
    }?;
    snapshot.visible_assets.get(selected_index)
}

pub(super) fn has_asset_selection(snapshot: &AssetWorkspaceSnapshot) -> bool {
    snapshot.selection.uuid.is_some()
        || !snapshot.selection.display_name.is_empty()
        || !snapshot.selection.locator.is_empty()
        || selected_asset(snapshot).is_some()
}

pub(super) fn selection_display_name(
    selection: &AssetSelectionSnapshot,
    selected_asset: Option<&AssetItemSnapshot>,
) -> String {
    if !selection.display_name.is_empty() {
        selection.display_name.clone()
    } else if let Some(asset) = selected_asset {
        asset.display_name.clone()
    } else {
        "No Asset Selected".to_string()
    }
}

pub(super) fn selection_locator(
    selection: &AssetSelectionSnapshot,
    selected_asset: Option<&AssetItemSnapshot>,
) -> String {
    if !selection.locator.is_empty() {
        selection.locator.clone()
    } else if let Some(asset) = selected_asset {
        asset.locator.clone()
    } else {
        "Select an asset to inspect".to_string()
    }
}

pub(super) fn selection_kind_label<'a>(
    selection: &'a AssetSelectionSnapshot,
    selected_asset: Option<&'a AssetItemSnapshot>,
) -> &'a str {
    if !selection.asset_type.display_name.is_empty() {
        selection.asset_type.display_name.as_str()
    } else if let Some(asset) = selected_asset {
        asset.asset_type.display_name.as_str()
    } else {
        "Unknown Type"
    }
}

pub(super) fn selection_identity(
    selection: &AssetSelectionSnapshot,
    selected_asset: Option<&AssetItemSnapshot>,
) -> String {
    selection
        .uuid
        .clone()
        .or_else(|| selected_asset.map(|asset| asset.uuid.clone()))
        .unwrap_or_else(|| "No UUID".to_string())
}

pub(super) fn selection_revision_label(
    selection: &AssetSelectionSnapshot,
    selected_asset: Option<&AssetItemSnapshot>,
) -> String {
    selection
        .resource_revision
        .or_else(|| selected_asset.and_then(|asset| asset.resource_revision))
        .map(|revision| format!("Revision {revision}"))
        .unwrap_or_else(|| "Revision pending".to_string())
}

pub(super) fn selection_diagnostics_text(
    selection: &AssetSelectionSnapshot,
    selected_asset: Option<&AssetItemSnapshot>,
) -> String {
    let diagnostics = if !selection.diagnostics.is_empty() {
        &selection.diagnostics
    } else if let Some(asset) = selected_asset {
        &asset.diagnostics
    } else {
        return "No active diagnostics".to_string();
    };

    if diagnostics.is_empty() {
        "No active diagnostics".to_string()
    } else {
        diagnostics.join("\n")
    }
}

pub(super) fn selection_metadata_summary(selection: &AssetSelectionSnapshot) -> String {
    let mut parts = Vec::new();
    parts.push(if selection.toolkit_view_id.is_empty() {
        "No toolkit".to_string()
    } else {
        selection.toolkit_view_id.clone()
    });
    if !selection.asset_unit.is_empty() {
        parts.push(format!("unit {}", selection.asset_unit));
    }
    if let Some(package_id) = selection.package_id.as_ref() {
        parts.push(format!("package {package_id}"));
    }
    if !selection.included_files.is_empty() {
        parts.push(format!("{} included", selection.included_files.len()));
    }
    if !selection.subassets.is_empty() {
        parts.push(format!("{} subassets", selection.subassets.len()));
    }
    parts.join(" | ")
}

pub(super) fn selection_metadata_body(
    selection: &AssetSelectionSnapshot,
    diagnostics: &str,
) -> String {
    let mut lines = Vec::new();
    if selection.diagnostics.is_empty() {
        lines.push("No active diagnostics".to_string());
    } else {
        lines.push(diagnostics.to_string());
    }
    if !selection.included_files.is_empty() {
        lines.push("Included files:".to_string());
        lines.extend(
            selection
                .included_files
                .iter()
                .map(|file| format!("- {file}")),
        );
    }
    if !selection.subassets.is_empty() {
        lines.push("Subassets:".to_string());
        lines.extend(selection.subassets.iter().map(|subasset| {
            format!(
                "- {} {} ({})",
                if subasset.asset_type.display_name.is_empty() {
                    "Unknown"
                } else {
                    subasset.asset_type.display_name.as_str()
                },
                subasset.locator,
                subasset.uuid
            )
        }));
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folder(
        folder_id: &str,
        parent_folder_id: Option<&str>,
        display_name: &str,
        asset_count: usize,
    ) -> AssetFolderSnapshot {
        AssetFolderSnapshot {
            folder_id: folder_id.to_string(),
            parent_folder_id: parent_folder_id.map(str::to_string),
            display_name: display_name.to_string(),
            recursive_asset_count: asset_count,
            ..AssetFolderSnapshot::default()
        }
    }

    #[test]
    fn selected_folder_breadcrumb_keeps_the_project_tree_and_asset_count() {
        let snapshot = AssetWorkspaceSnapshot {
            selected_folder_id: Some("res://models/props".to_string()),
            folder_tree: vec![
                folder("res://", None, "Content", 12),
                folder("res://models", Some("res://"), "Models", 8),
                folder("res://models/props", Some("res://models"), "Props", 5),
            ],
            ..AssetWorkspaceSnapshot::default()
        };

        assert_eq!(
            selected_folder_breadcrumb(&snapshot).as_deref(),
            Some("Content / Models / Props / 5 assets")
        );
    }

    #[test]
    fn selected_folder_breadcrumb_does_not_loop_on_a_cyclic_catalog() {
        let snapshot = AssetWorkspaceSnapshot {
            selected_folder_id: Some("res://loop-a".to_string()),
            folder_tree: vec![
                folder("res://loop-a", Some("res://loop-b"), "A", 1),
                folder("res://loop-b", Some("res://loop-a"), "B", 2),
            ],
            ..AssetWorkspaceSnapshot::default()
        };

        let breadcrumb = selected_folder_breadcrumb(&snapshot).expect("selected folder path");
        assert!(breadcrumb.ends_with(" / 1 assets"));
        assert!(breadcrumb.len() < 64);
    }

    #[test]
    fn selected_folder_breadcrumb_preserves_selection_and_parent_source_priority() {
        let snapshot = AssetWorkspaceSnapshot {
            selected_folder_id: Some("res://models/props".to_string()),
            folder_tree: vec![
                folder("res://", None, "Content", 12),
                folder("res://models", Some("res://"), "Tree Models", 8),
                folder("res://models/props", Some("res://models"), "Tree Props", 5),
            ],
            visible_folders: vec![
                folder("res://models", Some("res://"), "Visible Models", 8),
                folder(
                    "res://models/props",
                    Some("res://models"),
                    "Visible Props",
                    7,
                ),
            ],
            ..AssetWorkspaceSnapshot::default()
        };

        assert_eq!(
            selected_folder_breadcrumb(&snapshot).as_deref(),
            Some("Content / Tree Models / Visible Props / 7 assets")
        );
    }
}
