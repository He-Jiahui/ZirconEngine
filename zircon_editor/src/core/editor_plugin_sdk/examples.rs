use crate::core::editor_extension::{
    AssetEditorDescriptor, AssetImporterDescriptor, ComponentDrawerDescriptor,
    EditorExtensionRegistry, EditorExtensionRegistryError, EditorMenuItemDescriptor,
    EditorUiTemplateDescriptor, ViewDescriptor,
};
use crate::core::editor_operation::{
    EditorOperationDescriptor, EditorOperationPath, UndoableEditorOperation,
};
use crate::core::editor_plugin::{EditorPlugin, EditorPluginDescriptor};

#[derive(Clone, Debug)]
pub struct ExampleWindowEditorPlugin {
    descriptor: EditorPluginDescriptor,
}

impl Default for ExampleWindowEditorPlugin {
    fn default() -> Self {
        Self {
            descriptor: EditorPluginDescriptor::new(
                "sdk_example_window",
                "SDK Example Window",
                "zircon_editor_sdk_example_window",
            )
            .with_capability("editor.extension.sdk_example_window"),
        }
    }
}

impl EditorPlugin for ExampleWindowEditorPlugin {
    fn descriptor(&self) -> &EditorPluginDescriptor {
        &self.descriptor
    }

    fn register_editor_extensions(
        &self,
        registry: &mut EditorExtensionRegistry,
    ) -> Result<(), EditorExtensionRegistryError> {
        let operation_path = parse_operation("Sdk.Example.ToggleWeatherWindow")?;
        registry.register_operation(
            EditorOperationDescriptor::new(operation_path.clone(), "Toggle SDK Weather Window")
                .with_menu_path("Tools/SDK Examples/Toggle Weather Window"),
        )?;
        registry.register_view(ViewDescriptor::new(
            "sdk.example.weather_window",
            "SDK Weather",
            "SDK Examples",
        ))?;
        registry.register_menu_item(EditorMenuItemDescriptor::new(
            "Tools/SDK Examples/Toggle Weather Window",
            operation_path,
        ))
    }
}

#[derive(Clone, Debug)]
pub struct ExampleAssetInspectorPlugin {
    descriptor: EditorPluginDescriptor,
}

impl Default for ExampleAssetInspectorPlugin {
    fn default() -> Self {
        Self {
            descriptor: EditorPluginDescriptor::new(
                "sdk_example_asset",
                "SDK Example Asset Tools",
                "zircon_editor_sdk_example_asset",
            )
            .with_capability("editor.extension.sdk_example_asset"),
        }
    }
}

impl EditorPlugin for ExampleAssetInspectorPlugin {
    fn descriptor(&self) -> &EditorPluginDescriptor {
        &self.descriptor
    }

    fn register_editor_extensions(
        &self,
        registry: &mut EditorExtensionRegistry,
    ) -> Result<(), EditorExtensionRegistryError> {
        let import_operation = parse_operation("Sdk.Example.ImportModel")?;
        let open_operation = parse_operation("Sdk.Example.OpenModelInspector")?;
        registry.register_operation(
            EditorOperationDescriptor::new(import_operation.clone(), "Import SDK Model")
                .with_menu_path("Assets/SDK Examples/Import Model")
                .with_undoable(UndoableEditorOperation::new("Import SDK Model")),
        )?;
        registry.register_operation(EditorOperationDescriptor::new(
            open_operation.clone(),
            "Open SDK Model Inspector",
        ))?;
        registry.register_view(ViewDescriptor::new(
            "sdk.example.asset_inspector",
            "SDK Asset Inspector",
            "SDK Examples",
        ))?;
        registry.register_asset_importer(
            AssetImporterDescriptor::new(
                "sdk.example.asset.model_importer",
                "SDK Model Importer",
                import_operation,
            )
            .with_source_extension("glb")
            .with_source_extension("gltf")
            .with_output_kind("Model"),
        )?;
        registry.register_asset_editor(AssetEditorDescriptor::new(
            "Model",
            "sdk.example.asset_inspector",
            "SDK Model Inspector",
            open_operation,
        ))?;
        registry.register_ui_template(EditorUiTemplateDescriptor::new(
            "sdk.example.asset_inspector",
            "asset://sdk_examples/editor/model_inspector.ui.toml",
        ))?;
        registry.register_component_drawer(ComponentDrawerDescriptor::new(
            "sdk.example.ModelImportSettings",
            "asset://sdk_examples/editor/model_import_settings.ui.toml",
            "sdk.example.ModelImportSettingsController",
        ))
    }
}

fn parse_operation(path: &str) -> Result<EditorOperationPath, EditorExtensionRegistryError> {
    EditorOperationPath::parse(path).map_err(EditorExtensionRegistryError::Operation)
}
