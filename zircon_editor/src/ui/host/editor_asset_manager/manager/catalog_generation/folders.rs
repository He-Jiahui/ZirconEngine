use std::collections::{BTreeMap, HashMap, HashSet};

use zircon_runtime::asset::AssetUuid;
use zircon_runtime_interface::resource::ResourceScheme;

use crate::ui::host::editor_asset_manager::{AssetCatalogRecord, EditorAssetFolderRecord};

#[derive(Clone, Debug, Default)]
struct FolderBuilder {
    parent_folder_id: Option<String>,
    locator_prefix: String,
    display_name: String,
    child_folder_ids: HashSet<String>,
    direct_asset_uuids: Vec<String>,
    recursive_asset_count: usize,
}

pub(super) fn build_folder_records(
    catalog_by_uuid: &HashMap<AssetUuid, AssetCatalogRecord>,
) -> Vec<EditorAssetFolderRecord> {
    let mut folders = BTreeMap::<String, FolderBuilder>::new();
    folders.insert(
        "res://".to_string(),
        FolderBuilder {
            parent_folder_id: None,
            locator_prefix: "res://".to_string(),
            display_name: "Assets".to_string(),
            ..FolderBuilder::default()
        },
    );

    for record in catalog_by_uuid.values().filter(|record| {
        matches!(
            record.locator.scheme(),
            ResourceScheme::Res | ResourceScheme::Package
        )
    }) {
        let Some((root_id, root_display_name, asset_path)) = folder_root_for_record(record) else {
            continue;
        };
        folders
            .entry(root_id.clone())
            .or_insert_with(|| FolderBuilder {
                parent_folder_id: None,
                locator_prefix: root_id.clone(),
                display_name: root_display_name,
                ..FolderBuilder::default()
            });

        let folder_path = asset_path
            .rsplit_once('/')
            .map(|(folder_path, _)| folder_path)
            .unwrap_or_default();
        if let Some(terminal_folder_id) = terminal_folder_id(&root_id, folder_path) {
            if let Some(folder) = folders.get_mut(&terminal_folder_id) {
                folder
                    .direct_asset_uuids
                    .push(record.asset_uuid.to_string());
                folder.recursive_asset_count += 1;
                continue;
            }
        }
        let mut parent_id = root_id;
        for segment in folder_path.split('/').filter(|segment| !segment.is_empty()) {
            let folder_id = if parent_id == "res://" {
                format!("res://{segment}")
            } else {
                format!("{parent_id}/{segment}")
            };
            folders
                .entry(folder_id.clone())
                .or_insert_with(|| FolderBuilder {
                    parent_folder_id: Some(parent_id.clone()),
                    locator_prefix: folder_id.clone(),
                    display_name: segment.to_string(),
                    ..FolderBuilder::default()
                });
            if let Some(parent) = folders.get_mut(&parent_id) {
                let _ = parent.child_folder_ids.insert(folder_id.clone());
            }
            parent_id = folder_id;
        }
        if let Some(folder) = folders.get_mut(&parent_id) {
            folder
                .direct_asset_uuids
                .push(record.asset_uuid.to_string());
            folder.recursive_asset_count += 1;
        }
    }

    let mut ids_by_depth = folders
        .keys()
        .filter(|folder_id| folder_id.as_str() != "res://")
        .cloned()
        .collect::<Vec<_>>();
    ids_by_depth.sort_by_key(|folder_id| std::cmp::Reverse(folder_id.matches('/').count()));
    for folder_id in ids_by_depth {
        let count = folders
            .get(&folder_id)
            .map(|folder| folder.recursive_asset_count)
            .unwrap_or_default();
        let parent_id = folders
            .get(&folder_id)
            .and_then(|folder| folder.parent_folder_id.clone());
        if let Some(parent_id) = parent_id {
            if let Some(parent) = folders.get_mut(&parent_id) {
                parent.recursive_asset_count += count;
            }
        }
    }

    let asset_names = catalog_by_uuid
        .values()
        .map(|record| (record.asset_uuid.to_string(), record.display_name.as_str()))
        .collect::<HashMap<_, _>>();
    for folder in folders.values_mut() {
        folder.direct_asset_uuids.sort_by(|left, right| {
            let left_key = asset_names.get(left).copied().unwrap_or_default();
            let right_key = asset_names.get(right).copied().unwrap_or_default();
            left_key.cmp(right_key).then(left.cmp(right))
        });
    }

    folders
        .into_iter()
        .map(|(folder_id, folder)| {
            let child_folder_ids = ordered_child_folder_ids(folder.child_folder_ids);
            EditorAssetFolderRecord {
                folder_id,
                parent_folder_id: folder.parent_folder_id,
                locator_prefix: folder.locator_prefix,
                display_name: folder.display_name,
                child_folder_ids,
                direct_asset_uuids: folder.direct_asset_uuids,
                recursive_asset_count: folder.recursive_asset_count,
            }
        })
        .collect()
}

fn ordered_child_folder_ids(child_folder_ids: HashSet<String>) -> Vec<String> {
    let mut child_folder_ids = child_folder_ids.into_iter().collect::<Vec<_>>();
    // Siblings share a complete parent ID prefix, and each display name is its final segment.
    // Borrowing the final segment avoids copying a name index or comparing deep parent paths.
    child_folder_ids.sort_unstable_by(|left, right| {
        folder_display_name(left)
            .cmp(folder_display_name(right))
            .then(left.cmp(right))
    });
    child_folder_ids
}

fn folder_display_name(folder_id: &str) -> &str {
    folder_id
        .rsplit_once('/')
        .map(|(_, display_name)| display_name)
        .unwrap_or(folder_id)
}

fn terminal_folder_id(root_id: &str, folder_path: &str) -> Option<String> {
    if folder_path.is_empty() {
        None
    } else if root_id == "res://" {
        Some(format!("res://{folder_path}"))
    } else {
        Some(format!("{root_id}/{folder_path}"))
    }
}

fn folder_root_for_record(record: &AssetCatalogRecord) -> Option<(String, String, &str)> {
    match record.locator.scheme() {
        ResourceScheme::Res => Some((
            "res://".to_string(),
            "Assets".to_string(),
            record.locator.path(),
        )),
        ResourceScheme::Package => {
            let package_id = record.locator.package_id()?;
            let package_path = record.locator.package_path()?;
            Some((
                format!("package://{package_id}"),
                package_id.to_string(),
                package_path,
            ))
        }
        _ => None,
    }
}

#[cfg(test)]
#[path = "tests/folders_performance_tests.rs"]
mod performance_tests;
