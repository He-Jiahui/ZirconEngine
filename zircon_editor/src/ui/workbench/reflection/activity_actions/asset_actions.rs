use zircon_runtime_interface::ui::{
    binding::UiEventKind, event_ui::UiActionDescriptor, event_ui::UiParameterDescriptor,
    event_ui::UiValueType,
};

pub(super) const ASSET_ACTION_COUNT: usize = 2;

pub(super) fn asset_actions() -> [UiActionDescriptor; ASSET_ACTION_COUNT] {
    [
        UiActionDescriptor::new(
            "workbench.asset.mesh_import.path.set",
            UiEventKind::Change,
            "DraftCommand.SetMeshImportPath",
        )
        .with_parameter(UiParameterDescriptor::new("value", UiValueType::String)),
        UiActionDescriptor::new(
            "workbench.asset.model.import",
            UiEventKind::Click,
            "AssetCommand.ImportModel",
        ),
    ]
}
