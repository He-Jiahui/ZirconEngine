use zircon_ui::{
    binding::UiEventKind, event_ui::UiActionDescriptor, event_ui::UiParameterDescriptor,
    event_ui::UiValueType,
};

pub(super) fn asset_actions() -> Vec<UiActionDescriptor> {
    vec![
        UiActionDescriptor::new(
            "set_mesh_import_path",
            UiEventKind::Change,
            "DraftCommand.SetMeshImportPath",
        )
        .with_parameter(UiParameterDescriptor::new("value", UiValueType::String)),
        UiActionDescriptor::new(
            "import_model",
            UiEventKind::Click,
            "AssetCommand.ImportModel",
        ),
    ]
}
