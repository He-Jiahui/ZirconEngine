use crate::ui::workbench::snapshot::{
    AssetItemSnapshot, AssetReferenceSnapshot, AssetWorkspaceSnapshot,
};
use zircon_runtime_interface::ui::component::{
    UiDragPayload, UiDragPayloadKind, UiDragSourceMetadata,
};

const ACTIVITY_ASSET_CONTENT_CONTROL_ID: &str = "AssetsActivityContentPanel";
const BROWSER_ASSET_CONTENT_CONTROL_ID: &str = "AssetBrowserContentPanel";
const ACTIVITY_ASSET_REFERENCE_LEFT_CONTROL_ID: &str = "AssetsActivityReferenceLeftPanel";
const ACTIVITY_ASSET_REFERENCE_RIGHT_CONTROL_ID: &str = "AssetsActivityReferenceRightPanel";
const BROWSER_ASSET_REFERENCE_LEFT_CONTROL_ID: &str = "AssetBrowserReferenceLeftPanel";
const BROWSER_ASSET_REFERENCE_RIGHT_CONTROL_ID: &str = "AssetBrowserReferenceRightPanel";

pub(super) fn asset_drag_payload_from_snapshot(
    surface_mode: &str,
    asset_uuid: &str,
    snapshot: &AssetWorkspaceSnapshot,
) -> Option<UiDragPayload> {
    snapshot
        .visible_assets
        .iter()
        .find(|asset| asset.uuid == asset_uuid)
        .map(|asset| asset_drag_payload_from_item(surface_mode, asset))
}

fn asset_drag_payload_from_item(surface_mode: &str, asset: &AssetItemSnapshot) -> UiDragPayload {
    let source_control_id = match surface_mode {
        "activity" => ACTIVITY_ASSET_CONTENT_CONTROL_ID,
        _ => BROWSER_ASSET_CONTENT_CONTROL_ID,
    };
    UiDragPayload::new(UiDragPayloadKind::Asset, asset.locator.clone()).with_source(
        UiDragSourceMetadata::asset(
            surface_mode,
            source_control_id,
            asset.uuid.clone(),
            asset.locator.clone(),
            asset.display_name.clone(),
            format!("{:?}", asset.kind),
            asset.extension.clone(),
        ),
    )
}

pub(super) fn asset_drag_payload_from_reference(
    surface_mode: &str,
    list_kind: &str,
    asset_uuid: &str,
    snapshot: &AssetWorkspaceSnapshot,
) -> Option<UiDragPayload> {
    reference_list_for_kind(list_kind, snapshot)?
        .iter()
        .find(|reference| reference.uuid == asset_uuid)
        .and_then(|reference| {
            asset_drag_payload_from_reference_item(surface_mode, list_kind, reference)
        })
}

fn reference_list_for_kind<'a>(
    list_kind: &str,
    snapshot: &'a AssetWorkspaceSnapshot,
) -> Option<&'a [AssetReferenceSnapshot]> {
    match list_kind {
        "references" => Some(&snapshot.selection.references),
        "used_by" => Some(&snapshot.selection.used_by),
        _ => None,
    }
}

fn asset_drag_payload_from_reference_item(
    surface_mode: &str,
    list_kind: &str,
    reference: &AssetReferenceSnapshot,
) -> Option<UiDragPayload> {
    if !reference.known_project_asset {
        return None;
    }
    let source_control_id = reference_control_id(surface_mode, list_kind)?;
    let source_surface = format!("{surface_mode}.{list_kind}");
    let asset_kind = reference
        .kind
        .as_ref()
        .map(|kind| format!("{kind:?}"))
        .unwrap_or_else(|| "Asset Reference".to_string());
    Some(
        UiDragPayload::new(UiDragPayloadKind::Asset, reference.locator.clone()).with_source(
            UiDragSourceMetadata::asset(
                source_surface,
                source_control_id,
                reference.uuid.clone(),
                reference.locator.clone(),
                reference.display_name.clone(),
                asset_kind,
                extension_from_locator(&reference.locator),
            ),
        ),
    )
}

fn reference_control_id(surface_mode: &str, list_kind: &str) -> Option<&'static str> {
    match (surface_mode, list_kind) {
        ("activity", "references") => Some(ACTIVITY_ASSET_REFERENCE_LEFT_CONTROL_ID),
        ("activity", "used_by") => Some(ACTIVITY_ASSET_REFERENCE_RIGHT_CONTROL_ID),
        ("browser", "references") => Some(BROWSER_ASSET_REFERENCE_LEFT_CONTROL_ID),
        ("browser", "used_by") => Some(BROWSER_ASSET_REFERENCE_RIGHT_CONTROL_ID),
        _ => None,
    }
}

fn extension_from_locator(locator: &str) -> String {
    let file_name = locator
        .rsplit(|ch| ch == '/' || ch == '\\')
        .next()
        .unwrap_or(locator);
    file_name
        .rsplit_once('.')
        .map(|(_, extension)| extension)
        .filter(|extension| !extension.is_empty())
        .unwrap_or_default()
        .to_string()
}
