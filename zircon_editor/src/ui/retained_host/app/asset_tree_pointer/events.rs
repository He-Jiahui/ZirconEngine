use super::super::{asset_drag_payload, callback_dispatch, RetainedEditorHost};
use crate::ui::retained_host::asset_pointer::AssetPointerTreeRoute;
use zircon_runtime::asset::{AssetUri, AssetUuid};
use zircon_runtime_interface::ui::component::UiDragPayload;
use zircon_runtime_interface::ui::layout::UiPoint;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn asset_tree_pointer_event(
        &mut self,
        surface_mode: &str,
        kind: i32,
        button: i32,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        if kind != 2 || button != 1 {
            return;
        }
        let Some(payload) = self.active_asset_drag_payload.take() else {
            return;
        };
        if !asset_drag_payload::is_asset_content_drag_payload(&payload) {
            return;
        }
        if !self.prepare_asset_tree_pointer_target(surface_mode, width, height) {
            return;
        }
        let route = self
            .asset_surface_pointer_state(surface_mode)
            .and_then(|surface| surface.tree_bridge.route_at(UiPoint::new(x, y)));
        let Some(AssetPointerTreeRoute::Folder { folder_id, .. }) = route else {
            return;
        };
        let (asset_uuid, target) = match relocation_request_for_drop(&payload, &folder_id) {
            Ok(request) => request,
            Err(error) => {
                self.set_status_line(error);
                return;
            }
        };
        match callback_dispatch::dispatch_asset_relocation(
            &self.runtime,
            asset_uuid.to_string(),
            target.to_string(),
        ) {
            Ok(effects) => self.apply_dispatch_effects(effects),
            Err(error) => self.set_status_line(error),
        }
    }
}

fn relocation_request_for_drop(
    payload: &UiDragPayload,
    folder_id: &str,
) -> Result<(AssetUuid, AssetUri), String> {
    let source = payload
        .source
        .as_ref()
        .ok_or_else(|| "asset move payload is missing source metadata".to_owned())?;
    let asset_uuid = source
        .asset_uuid
        .as_deref()
        .ok_or_else(|| "asset move payload is missing its stable UUID".to_owned())?
        .parse::<AssetUuid>()
        .map_err(|error| format!("asset move payload has an invalid UUID: {error}"))?;
    let source_locator = source
        .locator
        .as_deref()
        .ok_or_else(|| "asset move payload is missing its source locator".to_owned())?;
    let source_locator = AssetUri::parse(source_locator)
        .map_err(|error| format!("asset move source locator is invalid: {error}"))?;
    if source_locator.label().is_some() {
        return Err("subassets cannot be moved independently of their source asset".to_owned());
    }
    let folder = AssetUri::parse(folder_id)
        .map_err(|error| format!("asset move target folder is invalid: {error}"))?;
    if folder.label().is_some() || folder.scheme() != source_locator.scheme() {
        return Err("asset move target must be a folder in the source asset scheme".to_owned());
    }
    let file_name = source_locator
        .path()
        .rsplit('/')
        .next()
        .filter(|segment| !segment.is_empty())
        .ok_or_else(|| "asset move source locator has no file name".to_owned())?;
    let folder_path = folder.path().trim_end_matches('/');
    let target_path = if folder_path.is_empty() {
        file_name.to_owned()
    } else {
        format!("{folder_path}/{file_name}")
    };
    let target = AssetUri::new(folder.scheme(), target_path, None)
        .map_err(|error| format!("asset move target locator is invalid: {error}"))?;
    Ok((asset_uuid, target))
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime_interface::ui::component::{UiDragPayloadKind, UiDragSourceMetadata};

    fn asset_payload(uuid: AssetUuid, locator: &str) -> UiDragPayload {
        UiDragPayload::new(UiDragPayloadKind::Asset, locator).with_source(
            UiDragSourceMetadata::asset(
                "browser",
                "AssetBrowserContentPanel",
                uuid.to_string(),
                locator,
                "Cube",
                "Model",
                "zmodel",
            ),
        )
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
}
