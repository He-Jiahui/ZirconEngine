use crate::ui::workbench::snapshot::{AssetItemSnapshot, AssetWorkspaceSnapshot};
use zircon_runtime_interface::ui::component::{
    UiDragPayload, UiDragPayloadKind, UiDragSourceMetadata,
};

const ACTIVITY_ASSET_CONTENT_CONTROL_ID: &str = "AssetsActivityContentPanel";
const BROWSER_ASSET_CONTENT_CONTROL_ID: &str = "AssetBrowserContentPanel";

pub(super) fn is_asset_content_drag_payload(payload: &UiDragPayload) -> bool {
    payload.kind == UiDragPayloadKind::Asset
        && payload.source.as_ref().is_some_and(|source| {
            matches!(
                source.source_control_id.as_str(),
                ACTIVITY_ASSET_CONTENT_CONTROL_ID | BROWSER_ASSET_CONTENT_CONTROL_ID
            )
        })
}

pub(in crate::ui::retained_host::app) fn asset_drag_payload_from_snapshot(
    surface_mode: &str,
    asset_uuid: &str,
    snapshot: &AssetWorkspaceSnapshot,
) -> Option<UiDragPayload> {
    let index = snapshot.visible_assets.selected_index(asset_uuid)?;
    snapshot
        .visible_assets
        .get(index)
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
