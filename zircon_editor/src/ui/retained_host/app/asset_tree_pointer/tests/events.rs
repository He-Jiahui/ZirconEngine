use super::*;
use zircon_runtime_interface::ui::component::{UiDragPayloadKind, UiDragSourceMetadata};

fn asset_payload(uuid: AssetUuid, locator: &str) -> UiDragPayload {
    UiDragPayload::new(UiDragPayloadKind::Asset, locator).with_source(UiDragSourceMetadata::asset(
        "browser",
        "AssetBrowserContentPanel",
        uuid.to_string(),
        locator,
        "Cube",
        "Model",
        "zmodel",
    ))
}

#[test]
fn folder_drop_preserves_uuid_and_source_file_name() {
    let uuid = AssetUuid::from_stable_label("asset-folder-drop");
    let request = relocation_request_for_drop(
        &asset_payload(uuid.clone(), "res://models/cube.zmodel"),
        "res://environment/props",
    )
    .unwrap();

    assert_eq!(request.0, uuid);
    assert_eq!(request.1.to_string(), "res://environment/props/cube.zmodel");
}

#[test]
fn folder_drop_rejects_independent_subasset_moves() {
    let uuid = AssetUuid::from_stable_label("asset-subasset-folder-drop");
    let error = relocation_request_for_drop(
        &asset_payload(uuid, "res://models/cube.zmodel#mesh"),
        "res://environment",
    )
    .unwrap_err();

    assert!(error.contains("subassets cannot be moved independently"));
}
