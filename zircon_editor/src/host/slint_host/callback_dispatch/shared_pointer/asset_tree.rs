use zircon_ui::{UiBindingValue, UiEventKind, UiPoint};

use crate::editor_event::EditorEventRuntime;
use crate::host::slint_host::{
    asset_pointer::{
        AssetFolderTreePointerBridge, AssetFolderTreePointerDispatch, AssetPointerTreeRoute,
    },
    event_bridge::SlintDispatchEffects,
};

use super::super::{BuiltinAssetSurfaceTemplateBridge, dispatch_builtin_asset_surface_control};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SharedAssetTreePointerClickDispatch {
    pub pointer: AssetFolderTreePointerDispatch,
    pub effects: Option<SlintDispatchEffects>,
}

pub(crate) fn dispatch_shared_asset_tree_pointer_click(
    runtime: &EditorEventRuntime,
    bridge: &BuiltinAssetSurfaceTemplateBridge,
    pointer_bridge: &mut AssetFolderTreePointerBridge,
    point: UiPoint,
) -> Result<SharedAssetTreePointerClickDispatch, String> {
    let pointer = pointer_bridge.handle_click(point)?;
    let effects = match pointer.route.as_ref() {
        Some(AssetPointerTreeRoute::Folder { folder_id, .. }) => {
            dispatch_builtin_asset_surface_control(
                runtime,
                bridge,
                "SelectFolder",
                UiEventKind::Change,
                vec![UiBindingValue::string(folder_id.as_str())],
            )
            .transpose()?
        }
        _ => None,
    };
    Ok(SharedAssetTreePointerClickDispatch { pointer, effects })
}
