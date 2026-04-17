use zircon_ui::{UiBindingValue, UiEventKind, UiPoint};

use crate::editor_event::EditorEventRuntime;
use crate::host::slint_host::{
    asset_pointer::{
        AssetPointerReferenceRoute, AssetReferenceListPointerBridge,
        AssetReferenceListPointerDispatch,
    },
    event_bridge::SlintDispatchEffects,
};

use super::super::{BuiltinAssetSurfaceTemplateBridge, dispatch_builtin_asset_surface_control};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SharedAssetReferencePointerClickDispatch {
    pub pointer: AssetReferenceListPointerDispatch,
    pub effects: Option<SlintDispatchEffects>,
}

pub(crate) fn dispatch_shared_asset_reference_pointer_click(
    runtime: &EditorEventRuntime,
    bridge: &BuiltinAssetSurfaceTemplateBridge,
    pointer_bridge: &mut AssetReferenceListPointerBridge,
    point: UiPoint,
) -> Result<SharedAssetReferencePointerClickDispatch, String> {
    let pointer = pointer_bridge.handle_click(point)?;
    let effects = match pointer.route.as_ref() {
        Some(AssetPointerReferenceRoute::Item { asset_uuid, .. }) => {
            dispatch_builtin_asset_surface_control(
                runtime,
                bridge,
                "ActivateReference",
                UiEventKind::Click,
                vec![UiBindingValue::string(asset_uuid.as_str())],
            )
            .transpose()?
        }
        _ => None,
    };
    Ok(SharedAssetReferencePointerClickDispatch { pointer, effects })
}
