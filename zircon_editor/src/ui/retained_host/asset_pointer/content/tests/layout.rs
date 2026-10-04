use super::*;
use crate::ui::workbench::snapshot::{AssetFolderSnapshot, AssetWorkspaceSnapshot};

#[test]
fn browser_content_layout_keeps_source_tree_folders_out_of_asset_rows() {
    let snapshot = AssetWorkspaceSnapshot {
        visible_folders: vec![AssetFolderSnapshot {
            folder_id: "res://materials".to_string(),
            ..AssetFolderSnapshot::default()
        }],
        ..AssetWorkspaceSnapshot::default()
    };

    let browser = AssetContentListPointerLayout::from_snapshot(
        &snapshot,
        UiSize::new(420.0, 220.0),
        AssetContentSurfaceProfile::Browser,
    );
    let activity = AssetContentListPointerLayout::from_snapshot(
        &snapshot,
        UiSize::new(420.0, 220.0),
        AssetContentSurfaceProfile::Activity,
    );

    assert!(browser.folder_ids.is_empty());
    assert_eq!(activity.folder_ids, vec!["res://materials"]);
}
