use crate::core::asset::{AssetToolkitDescriptor, AssetTypeContribution, AssetTypeId};
use crate::core::commands::EditorCommandDescriptor;
use crate::core::editor_extension::{
    AssetImporterDescriptor, ComponentDrawerDescriptor, EditorExtensionRegistry,
    EditorExtensionRegistryError, EditorMenuItemDescriptor, EditorUiTemplateDescriptor,
    ViewDescriptor,
};
use crate::core::editor_operation::EditorOperationPath;
use crate::core::editor_plugin::{EditorPlugin, EditorPluginDescriptor};
use zircon_runtime_interface::resource::ResourceKind;

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
        let operation_path = parse_operation("sdk.example.toggle_weather_window")?;
        registry.register_command(
            EditorCommandDescriptor::operation(operation_path.clone(), "Toggle SDK Weather Window")
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
        let import_operation = parse_operation("sdk.example.import_model")?;
        let open_operation = parse_operation("sdk.example.open_model_inspector")?;
        registry.register_command(
            EditorCommandDescriptor::operation(import_operation.clone(), "Import SDK Model")
                .with_menu_path("Assets/SDK Examples/Import Model"),
        )?;
        registry.register_command(EditorCommandDescriptor::operation(
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
            .with_output_type(AssetTypeId::from_resource_kind(ResourceKind::Model)),
        )?;
        registry.register_asset_type_contribution(
            AssetTypeContribution::augment(AssetTypeId::from_resource_kind(ResourceKind::Model))
                .with_toolkit(AssetToolkitDescriptor::new(
                    "sdk.example.asset_inspector",
                    open_operation,
                )),
        )?;
        registry.register_ui_template(EditorUiTemplateDescriptor::new(
            "sdk.example.asset_inspector",
            "asset://sdk_examples/editor/model_inspector.zui",
        ))?;
        registry.register_component_drawer(ComponentDrawerDescriptor::new(
            "sdk.example.ModelImportSettings",
            "asset://sdk_examples/editor/model_import_settings.zui",
            "sdk.example.ModelImportSettingsController",
        ))
    }
}

fn parse_operation(path: &str) -> Result<EditorOperationPath, EditorExtensionRegistryError> {
    EditorOperationPath::parse(path).map_err(EditorExtensionRegistryError::OperationPath)
}
