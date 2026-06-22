use zircon_editor::core::editor_authoring_extension::AssetCreationTemplateDescriptor;
use zircon_editor::core::editor_extension::{
    AssetEditorDescriptor, AssetImporterDescriptor, ComponentDrawerDescriptor,
    EditorExtensionRegistry, EditorExtensionRegistryError, EditorMenuItemDescriptor,
    EditorUiTemplateDescriptor, ViewDescriptor,
};
use zircon_editor::core::editor_operation::{
    EditorOperationDescriptor, EditorOperationPath, UndoableEditorOperation,
};

use crate::capability::{ASSET_FIXTURE_CAPABILITY, WINDOW_CAPABILITY};
use crate::extension_ids::{
    ASSET_INSPECTOR_VIEW_ID, MODEL_ASSET_KIND, MODEL_IMPORTER_ID, MODEL_IMPORT_SETTINGS_COMPONENT,
    MODEL_IMPORT_SETTINGS_TEMPLATE_ID, WINDOW_VIEW_ID,
};

pub(crate) fn register_example_window(
    registry: &mut EditorExtensionRegistry,
) -> Result<(), EditorExtensionRegistryError> {
    let operation_path = parse_operation("Sdk.Examples.ToggleWeatherWindow")?;
    registry.register_operation(
        EditorOperationDescriptor::new(operation_path.clone(), "Toggle SDK Weather Window")
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
    let import_operation = parse_operation("Sdk.Examples.ImportModel")?;
    let open_operation = parse_operation("Sdk.Examples.OpenModelInspector")?;
    let create_settings_operation = parse_operation("Sdk.Examples.CreateModelImportSettings")?;

    registry.register_operation(
        EditorOperationDescriptor::new(import_operation.clone(), "Import SDK Model")
            .with_menu_path("Assets/SDK Examples/Import Model")
            .with_undoable(UndoableEditorOperation::new("Import SDK Model"))
            .with_required_capabilities([ASSET_FIXTURE_CAPABILITY]),
    )?;
    registry.register_menu_item(
        EditorMenuItemDescriptor::new("Assets/SDK Examples/Import Model", import_operation.clone())
            .with_required_capabilities([ASSET_FIXTURE_CAPABILITY]),
    )?;
    registry.register_operation(
        EditorOperationDescriptor::new(open_operation.clone(), "Open SDK Model Inspector")
            .with_required_capabilities([ASSET_FIXTURE_CAPABILITY]),
    )?;
    registry.register_operation(
        EditorOperationDescriptor::new(
            create_settings_operation.clone(),
            "Create SDK Model Import Settings",
        )
        .with_menu_path("Assets/Create/SDK Examples/Model Import Settings")
        .with_undoable(UndoableEditorOperation::new(
            "Create SDK Model Import Settings",
        ))
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
            .with_output_kind(MODEL_ASSET_KIND)
            .with_priority(10)
            .with_required_capabilities([ASSET_FIXTURE_CAPABILITY]),
    )?;
    registry.register_asset_editor(
        AssetEditorDescriptor::new(
            MODEL_ASSET_KIND,
            ASSET_INSPECTOR_VIEW_ID,
            "SDK Model Inspector",
            open_operation.clone(),
        )
        .with_required_capabilities([ASSET_FIXTURE_CAPABILITY]),
    )?;
    registry.register_ui_template(EditorUiTemplateDescriptor::new(
        ASSET_INSPECTOR_VIEW_ID,
        "asset://plugin_sdk_examples/editor/model_inspector.zui",
    ))?;
    registry.register_ui_template(EditorUiTemplateDescriptor::new(
        MODEL_IMPORT_SETTINGS_TEMPLATE_ID,
        "asset://plugin_sdk_examples/editor/model_import_settings.zui",
    ))?;
    registry.register_asset_creation_template(
        AssetCreationTemplateDescriptor::new(
            MODEL_IMPORT_SETTINGS_TEMPLATE_ID,
            "SDK Model Import Settings",
            "ModelImportSettings",
            create_settings_operation,
        )
        .with_default_document("asset://plugin_sdk_examples/examples/model_import_settings.toml")
        .with_required_capabilities([ASSET_FIXTURE_CAPABILITY]),
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
    EditorOperationPath::parse(path).map_err(EditorExtensionRegistryError::Operation)
}
