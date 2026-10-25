use zircon_runtime_interface::ui::{
    binding::{UiBindingValue, UiEventKind},
    layout::UiPoint,
};

use crate::core::editor_event::{
    EditorAssetEvent, EditorEvent, EditorEventEnvelope, EditorEventSource,
};
use crate::ui::host::EditorHostEventController;
use crate::ui::retained_host::{
    asset_pointer::{
        AssetFolderTreePointerBridge, AssetFolderTreePointerDispatch, AssetPointerTreeRoute,
    },
    event_bridge::UiHostEventEffects,
};

use super::super::common::dispatch_envelope;
use super::super::{dispatch_builtin_asset_surface_control, BuiltinAssetSurfaceTemplateBridge};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SharedAssetTreePointerClickDispatch {
    pub pointer: AssetFolderTreePointerDispatch,
    pub effects: Option<UiHostEventEffects>,
}

pub(crate) fn dispatch_shared_asset_tree_pointer_click(
    runtime: &EditorHostEventController,
    bridge: &BuiltinAssetSurfaceTemplateBridge,
    pointer_bridge: &mut AssetFolderTreePointerBridge,
    point: UiPoint,
) -> Result<SharedAssetTreePointerClickDispatch, String> {
    let pointer = pointer_bridge.handle_click(point)?;
    let effects = match pointer.route.as_ref() {
        Some(AssetPointerTreeRoute::Disclosure {
            folder_id,
            expanded,
            expected_catalog_revision,
            ..
        }) => Some(dispatch_envelope(
            runtime,
            EditorEventEnvelope::new(
                EditorEventSource::RetainedHost,
                EditorEvent::Asset(EditorAssetEvent::SetFolderExpanded {
                    folder_id: folder_id.clone(),
                    expanded: *expanded,
                    expected_catalog_revision: *expected_catalog_revision,
                }),
            ),
        )?),
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
