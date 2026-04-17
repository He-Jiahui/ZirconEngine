use std::collections::{BTreeMap, HashMap};

use crate::AssetUriScheme;

use super::default_editor_asset_manager::EditorAssetState;
use super::super::EditorAssetFolderRecord;

#[derive(Clone, Debug, Default)]
struct FolderBuilder {
    parent_folder_id: Option<String>,
    locator_prefix: String,
    display_name: String,
    child_folder_ids: Vec<String>,
    direct_asset_uuids: Vec<String>,
    recursive_asset_count: usize,
}

pub(super) fn build_folder_records(state: &EditorAssetState) -> Vec<EditorAssetFolderRecord> {
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

    for record in state
        .catalog_by_uuid
        .values()
        .filter(|record| record.locator.scheme() == AssetUriScheme::Res)
    {
        let path_segments = record.locator.path().split('/').collect::<Vec<_>>();
        let folder_segments = if path_segments.len() > 1 {
            &path_segments[..path_segments.len() - 1]
        } else {
            &[][..]
        };
        let mut parent_id = "res://".to_string();
        for segment in folder_segments {
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
                    display_name: (*segment).to_string(),
                    ..FolderBuilder::default()
                });
            if let Some(parent) = folders.get_mut(&parent_id) {
                if !parent.child_folder_ids.contains(&folder_id) {
                    parent.child_folder_ids.push(folder_id.clone());
                }
            }
            parent_id = folder_id;
        }
        if let Some(folder) = folders.get_mut(&parent_id) {
            folder.direct_asset_uuids.push(record.asset_uuid.to_string());
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

    let folder_names = folders
        .iter()
        .map(|(id, folder)| (id.clone(), folder.display_name.clone()))
        .collect::<HashMap<_, _>>();
    let asset_names = state
        .catalog_by_uuid
        .values()
        .map(|record| (record.asset_uuid.to_string(), record.display_name.clone()))
        .collect::<HashMap<_, _>>();
    for folder in folders.values_mut() {
        folder.child_folder_ids.sort_by(|left, right| {
            folder_names[left]
                .cmp(&folder_names[right])
                .then(left.cmp(right))
        });
        folder.direct_asset_uuids.sort_by(|left, right| {
            let left_key = asset_names.get(left).cloned().unwrap_or_default();
            let right_key = asset_names.get(right).cloned().unwrap_or_default();
            left_key.cmp(&right_key).then(left.cmp(right))
        });
    }

    folders
        .into_iter()
        .map(|(folder_id, folder)| EditorAssetFolderRecord {
            folder_id,
            parent_folder_id: folder.parent_folder_id,
            locator_prefix: folder.locator_prefix,
            display_name: folder.display_name,
            child_folder_ids: folder.child_folder_ids,
            direct_asset_uuids: folder.direct_asset_uuids,
            recursive_asset_count: folder.recursive_asset_count,
        })
        .collect()
}
