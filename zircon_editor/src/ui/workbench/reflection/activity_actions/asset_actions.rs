use zircon_ui::{UiActionDescriptor, UiEventKind, UiParameterDescriptor, UiValueType};

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
