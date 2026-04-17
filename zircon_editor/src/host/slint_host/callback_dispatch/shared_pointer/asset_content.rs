use zircon_ui::{UiBindingValue, UiEventKind, UiPoint};

use crate::editor_event::EditorEventRuntime;
use crate::host::slint_host::{
    asset_pointer::{
        AssetContentListPointerBridge, AssetContentListPointerDispatch, AssetPointerContentRoute,
    },
    event_bridge::SlintDispatchEffects,
};

use super::super::{BuiltinAssetSurfaceTemplateBridge, dispatch_builtin_asset_surface_control};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SharedAssetContentPointerClickDispatch {
    pub pointer: AssetContentListPointerDispatch,
    pub effects: Option<SlintDispatchEffects>,
}

pub(crate) fn dispatch_shared_asset_content_pointer_click(
    runtime: &EditorEventRuntime,
    bridge: &BuiltinAssetSurfaceTemplateBridge,
    pointer_bridge: &mut AssetContentListPointerBridge,
    point: UiPoint,
) -> Result<SharedAssetContentPointerClickDispatch, String> {
    let pointer = pointer_bridge.handle_click(point)?;
    let effects = match pointer.route.as_ref() {
        Some(AssetPointerContentRoute::Folder { folder_id, .. }) => {
            dispatch_builtin_asset_surface_control(
                runtime,
                bridge,
                "SelectFolder",
                UiEventKind::Change,
                vec![UiBindingValue::string(folder_id.as_str())],
            )
            .transpose()?
        }
        Some(AssetPointerContentRoute::Item { asset_uuid, .. }) => {
            dispatch_builtin_asset_surface_control(
                runtime,
                bridge,
                "SelectItem",
                UiEventKind::Change,
                vec![UiBindingValue::string(asset_uuid.as_str())],
            )
            .transpose()?
        }
        _ => None,
    };
    Ok(SharedAssetContentPointerClickDispatch { pointer, effects })
}
