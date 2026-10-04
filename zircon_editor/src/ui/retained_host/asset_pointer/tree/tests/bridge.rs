use super::*;

#[test]
fn pane_size_patch_preserves_folder_projection() {
    let mut bridge = AssetFolderTreePointerBridge::new();
    let folder_ids = (0..1_024)
        .map(|index| format!("res://folder-{index}"))
        .collect::<Vec<_>>();
    bridge.sync(
        AssetFolderTreePointerLayout {
            pane_size: UiSize::new(240.0, 180.0),
            folder_ids: folder_ids.clone(),
        },
        AssetListPointerState::default(),
    );

    let state_change = bridge.sync_pane_size(UiSize::new(480.0, 360.0));

    assert!(state_change.is_none());
    assert_eq!(bridge.layout.folder_ids, folder_ids);
    assert_eq!(bridge.surface_node_count_for_test(), 2);
}
