use zircon_editor::core::asset::{
    AssetCreationTemplateDescriptor, AssetToolkitDescriptor, AssetTypeContribution, AssetTypeId,
    AssetTypePresentation, ThumbnailProviderDescriptor,
};
use zircon_editor::core::commands::EditorCommandDescriptor;
use zircon_editor::core::editor_extension::{
    AssetImporterDescriptor, ComponentDrawerDescriptor, EditorExtensionRegistry,
    EditorExtensionRegistryError, EditorMenuItemDescriptor, EditorUiTemplateDescriptor,
    ViewDescriptor,
};
use zircon_editor::core::editor_operation::EditorOperationPath;
use zircon_runtime_interface::resource::ResourceKind;

use crate::capability::{ASSET_FIXTURE_CAPABILITY, WINDOW_CAPABILITY};
use crate::extension_ids::{
    ASSET_INSPECTOR_VIEW_ID, MODEL_IMPORTER_ID, MODEL_IMPORT_SETTINGS_COMPONENT,
    MODEL_IMPORT_SETTINGS_TEMPLATE_ID, WINDOW_VIEW_ID,
};

pub(crate) fn register_example_window(
    registry: &mut EditorExtensionRegistry,
) -> Result<(), EditorExtensionRegistryError> {
    let operation_path = parse_operation("sdk.examples.toggle_weather_window")?;
    registry.register_command(
        EditorCommandDescriptor::operation(operation_path.clone(), "Toggle SDK Weather Window")
            .with_menu_path("Tools/SDK Examples/Toggle Weather Window")
            .with_required_capabilities([WINDOW_CAPABILITY]),
    )?;
    registry.register_menu_item(
        EditorMenuItemDescriptor::new("Tools/SDK Examples/Toggle Weather Window", operation_path)
            .with_required_capabilities([WINDOW_CAPABILITY]),
    )?;
    registry.register_view(ViewDescriptor::new(
        WINDOW_VIEW_ID,
        "SDK Weather",
        "SDK Examples",
    ))
}

pub(crate) fn register_importer_and_inspector(
    registry: &mut EditorExtensionRegistry,
) -> Result<(), EditorExtensionRegistryError> {
    let import_operation = parse_operation("sdk.examples.import_model")?;
    let open_operation = parse_operation("sdk.examples.open_model_inspector")?;
    let create_settings_operation = parse_operation("sdk.examples.create_model_import_settings")?;

    registry.register_command(
        EditorCommandDescriptor::operation(import_operation.clone(), "Import SDK Model")
            .with_menu_path("Assets/SDK Examples/Import Model")
            .with_required_capabilities([ASSET_FIXTURE_CAPABILITY]),
    )?;
    registry.register_menu_item(
        EditorMenuItemDescriptor::new("Assets/SDK Examples/Import Model", import_operation.clone())
            .with_required_capabilities([ASSET_FIXTURE_CAPABILITY]),
    )?;
    registry.register_command(
        EditorCommandDescriptor::operation(open_operation.clone(), "Open SDK Model Inspector")
            .with_required_capabilities([ASSET_FIXTURE_CAPABILITY]),
    )?;
    registry.register_command(
        EditorCommandDescriptor::operation(
            create_settings_operation.clone(),
            "Create SDK Model Import Settings",
        )
        .with_menu_path("Assets/Create/SDK Examples/Model Import Settings")
        .with_required_capabilities([ASSET_FIXTURE_CAPABILITY]),
    )?;
    registry.register_menu_item(
        EditorMenuItemDescriptor::new(
            "Assets/Create/SDK Examples/Model Import Settings",
            create_settings_operation.clone(),
        )
        .with_required_capabilities([ASSET_FIXTURE_CAPABILITY]),
    )?;
    registry.register_view(ViewDescriptor::new(
        ASSET_INSPECTOR_VIEW_ID,
        "SDK Asset Inspector",
        "SDK Examples",
    ))?;
    registry.register_asset_importer(
        AssetImporterDescriptor::new(MODEL_IMPORTER_ID, "SDK Model Importer", import_operation)
            .with_source_extensions(["glb", "gltf"])
            .with_output_type(AssetTypeId::from_resource_kind(ResourceKind::Model))
            .with_priority(10)
            .with_required_capabilities([ASSET_FIXTURE_CAPABILITY]),
    )?;
    registry.register_asset_type_contribution(
        AssetTypeContribution::augment(AssetTypeId::from_resource_kind(ResourceKind::Model))
            .with_toolkit(
                AssetToolkitDescriptor::new(ASSET_INSPECTOR_VIEW_ID, open_operation.clone())
                    .with_required_capabilities([ASSET_FIXTURE_CAPABILITY]),
            ),
    )?;
    registry.register_ui_template(EditorUiTemplateDescriptor::new(
        ASSET_INSPECTOR_VIEW_ID,
        "asset://plugin_sdk_examples/editor/model_inspector.zui",
    ))?;
    registry.register_ui_template(EditorUiTemplateDescriptor::new(
        MODEL_IMPORT_SETTINGS_TEMPLATE_ID,
        "asset://plugin_sdk_examples/editor/model_import_settings.zui",
    ))?;
    registry.register_asset_type_contribution(
        AssetTypeContribution::define(
            AssetTypeId::parse("model.import_settings")?,
            AssetTypePresentation::new(
                "Model Import Settings",
                "MIS",
                "asset-model-import-settings",
                "asset.model",
            ),
            ThumbnailProviderDescriptor::Icon("asset-model-import-settings".to_owned()),
        )
        .with_creation_template(
            AssetCreationTemplateDescriptor::new(
                MODEL_IMPORT_SETTINGS_TEMPLATE_ID,
                "SDK Model Import Settings",
                create_settings_operation,
            )
            .with_default_document(
                "asset://plugin_sdk_examples/examples/model_import_settings.toml",
            )
            .with_required_capabilities([ASSET_FIXTURE_CAPABILITY]),
        ),
    )?;
    registry.register_component_drawer(
        ComponentDrawerDescriptor::new(
            MODEL_IMPORT_SETTINGS_COMPONENT,
            "asset://plugin_sdk_examples/editor/model_import_settings.zui",
            "sdk.example.ModelImportSettingsController",
        )
        .with_binding(open_operation.as_str()),
    )
}

fn parse_operation(path: &str) -> Result<EditorOperationPath, EditorExtensionRegistryError> {
    EditorOperationPath::parse(path).map_err(EditorExtensionRegistryError::OperationPath)
}
