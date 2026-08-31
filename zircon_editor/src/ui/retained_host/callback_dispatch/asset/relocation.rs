use crate::ui::binding::{
    AssetCommand, EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind,
};
use crate::ui::host::EditorHostEventController;
use crate::ui::retained_host::event_bridge::UiHostEventEffects;

use super::super::common::dispatch_editor_binding;

pub(crate) fn dispatch_asset_relocation(
    runtime: &EditorHostEventController,
    asset_uuid: impl Into<String>,
    target_locator: impl Into<String>,
) -> Result<UiHostEventEffects, String> {
    dispatch_editor_binding(
        runtime,
        EditorUiBinding::new(
            "AssetTree",
            "RelocateAsset",
            EditorUiEventKind::Drop,
            EditorUiBindingPayload::asset_command(AssetCommand::RelocateAsset {
                asset_uuid: asset_uuid.into(),
                target_locator: target_locator.into(),
            }),
        ),
    )
}
