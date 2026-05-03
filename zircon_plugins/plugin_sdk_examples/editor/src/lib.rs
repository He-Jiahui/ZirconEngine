use zircon_editor::core::editor_authoring_extension::AssetCreationTemplateDescriptor;
use zircon_editor::core::editor_extension::{
    AssetEditorDescriptor, AssetImporterDescriptor, ComponentDrawerDescriptor,
    EditorExtensionRegistry, EditorExtensionRegistryError, EditorMenuItemDescriptor,
    EditorUiTemplateDescriptor, ViewDescriptor,
};
use zircon_editor::core::editor_operation::{
    EditorOperationDescriptor, EditorOperationPath, UndoableEditorOperation,
};
use zircon_runtime::{
    plugin::ExportPackagingStrategy, plugin::ExportTargetPlatform, plugin::PluginPackageManifest,
    RuntimeTargetMode,
};

pub const PLUGIN_ID: &str = "plugin_sdk_examples";
pub const CAPABILITY: &str = "editor.extension.plugin_sdk_examples";
pub const WINDOW_CAPABILITY: &str = "editor.extension.plugin_sdk_examples.window";
pub const ASSET_FIXTURE_CAPABILITY: &str = "editor.extension.plugin_sdk_examples.asset_fixture";

pub const WINDOW_VIEW_ID: &str = "sdk.example.weather_window";
pub const ASSET_INSPECTOR_VIEW_ID: &str = "sdk.example.asset_inspector";
pub const MODEL_IMPORTER_ID: &str = "sdk.example.asset.model_importer";
pub const MODEL_IMPORT_SETTINGS_COMPONENT: &str = "sdk.example.ModelImportSettings";
pub const MODEL_IMPORT_SETTINGS_TEMPLATE_ID: &str = "sdk.example.model_import_settings";
pub const MODEL_ASSET_KIND: &str = "Model";

#[derive(Clone, Debug)]
pub struct PluginSdkExamplesEditorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl PluginSdkExamplesEditorPlugin {
    pub fn new() -> Self {
        Self {
            descriptor: editor_plugin_descriptor(),
        }
    }
}

impl zircon_editor::EditorPlugin for PluginSdkExamplesEditorPlugin {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }

    fn register_editor_extensions(
        &self,
        registry: &mut EditorExtensionRegistry,
    ) -> Result<(), EditorExtensionRegistryError> {
        register_example_window(registry)?;
        register_importer_and_inspector(registry)
    }
}

#[derive(Clone, Debug)]
pub struct ExampleWindowEditorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl Default for ExampleWindowEditorPlugin {
    fn default() -> Self {
        Self {
            descriptor: zircon_editor::EditorPluginDescriptor::new(
                "plugin_sdk_examples.window",
                "SDK Example Window",
                "zircon_plugin_sdk_examples_editor",
            )
            .with_capability(WINDOW_CAPABILITY),
        }
    }
}

impl zircon_editor::EditorPlugin for ExampleWindowEditorPlugin {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }

    fn register_editor_extensions(
        &self,
        registry: &mut EditorExtensionRegistry,
    ) -> Result<(), EditorExtensionRegistryError> {
        register_example_window(registry)
    }
}

#[derive(Clone, Debug)]
pub struct ExampleAssetInspectorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl Default for ExampleAssetInspectorPlugin {
    fn default() -> Self {
        Self {
            descriptor: zircon_editor::EditorPluginDescriptor::new(
                "plugin_sdk_examples.asset",
                "SDK Example Asset Tools",
                "zircon_plugin_sdk_examples_editor",
            )
            .with_capability(ASSET_FIXTURE_CAPABILITY),
        }
    }
}

impl zircon_editor::EditorPlugin for ExampleAssetInspectorPlugin {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }

    fn register_editor_extensions(
        &self,
        registry: &mut EditorExtensionRegistry,
    ) -> Result<(), EditorExtensionRegistryError> {
        register_importer_and_inspector(registry)
    }
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(
        PLUGIN_ID,
        "Plugin SDK Examples",
        "zircon_plugin_sdk_examples_editor",
    )
    .with_capability(CAPABILITY)
    .with_capability(WINDOW_CAPABILITY)
    .with_capability(ASSET_FIXTURE_CAPABILITY)
}

pub fn editor_plugin() -> PluginSdkExamplesEditorPlugin {
    PluginSdkExamplesEditorPlugin::new()
}

pub fn package_manifest() -> PluginPackageManifest {
    zircon_editor::EditorPlugin::package_manifest(&editor_plugin(), base_package_manifest())
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_plugin()).to_vec()
}

pub fn plugin_registration() -> zircon_editor::EditorPluginRegistrationReport {
    zircon_editor::EditorPluginRegistrationReport::from_plugin(
        &editor_plugin(),
        base_package_manifest(),
    )
}

fn base_package_manifest() -> PluginPackageManifest {
    PluginPackageManifest::new(PLUGIN_ID, "Plugin SDK Examples")
        .with_sdk_api_version("0.1.0")
        .with_category("sdk")
        .with_supported_targets([RuntimeTargetMode::EditorHost])
        .with_supported_platforms([
            ExportTargetPlatform::Windows,
            ExportTargetPlatform::Linux,
            ExportTargetPlatform::Macos,
        ])
        .with_capabilities([CAPABILITY, WINDOW_CAPABILITY, ASSET_FIXTURE_CAPABILITY])
        .with_asset_root("assets")
        .with_content_root("examples")
        .with_default_packaging([
            ExportPackagingStrategy::SourceTemplate,
            ExportPackagingStrategy::LibraryEmbed,
        ])
}

fn register_example_window(
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

fn register_importer_and_inspector(
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
        "asset://plugin_sdk_examples/editor/model_inspector.ui.toml",
    ))?;
    registry.register_ui_template(EditorUiTemplateDescriptor::new(
        MODEL_IMPORT_SETTINGS_TEMPLATE_ID,
        "asset://plugin_sdk_examples/editor/model_import_settings.ui.toml",
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
            "asset://plugin_sdk_examples/editor/model_import_settings.ui.toml",
            "sdk.example.ModelImportSettingsController",
        )
        .with_binding(open_operation.as_str()),
    )
}

fn parse_operation(path: &str) -> Result<EditorOperationPath, EditorExtensionRegistryError> {
    EditorOperationPath::parse(path).map_err(EditorExtensionRegistryError::Operation)
}

#[cfg(test)]
mod tests {
    use zircon_runtime::plugin::PluginModuleKind;

    use super::*;

    #[test]
    fn sdk_examples_package_contributes_window_importer_and_inspector() {
        let registration = plugin_registration();

        assert!(registration.is_success(), "{:?}", registration.diagnostics);
        assert_eq!(
            registration.capabilities,
            vec![
                CAPABILITY.to_string(),
                WINDOW_CAPABILITY.to_string(),
                ASSET_FIXTURE_CAPABILITY.to_string()
            ]
        );
        assert!(registration
            .extensions
            .views()
            .iter()
            .any(|view| view.id() == WINDOW_VIEW_ID));
        assert!(registration
            .extensions
            .views()
            .iter()
            .any(|view| view.id() == ASSET_INSPECTOR_VIEW_ID));
        let importer = registration
            .extensions
            .asset_importers()
            .into_iter()
            .find(|importer| importer.id() == MODEL_IMPORTER_ID)
            .expect("SDK model importer");
        assert_eq!(
            importer.source_extensions(),
            &["glb".to_string(), "gltf".to_string()]
        );
        assert_eq!(importer.output_kind(), Some(MODEL_ASSET_KIND));
        assert_eq!(importer.priority(), 10);
        assert!(registration
            .extensions
            .asset_editors()
            .iter()
            .any(|editor| editor.asset_kind() == MODEL_ASSET_KIND
                && editor.view_id() == ASSET_INSPECTOR_VIEW_ID));
        assert!(registration
            .extensions
            .component_drawers()
            .iter()
            .any(|drawer| drawer.component_type() == MODEL_IMPORT_SETTINGS_COMPONENT));
        assert!(registration
            .extensions
            .asset_creation_templates()
            .iter()
            .any(|template| template.id() == MODEL_IMPORT_SETTINGS_TEMPLATE_ID));
    }

    #[test]
    fn sdk_examples_package_manifest_declares_sdk_fixture_metadata() {
        let manifest = package_manifest();

        assert_eq!(manifest.sdk_api_version, "0.1.0");
        assert_eq!(manifest.category, "sdk");
        assert_eq!(
            manifest.supported_targets,
            vec![RuntimeTargetMode::EditorHost]
        );
        assert!(manifest.capabilities.contains(&CAPABILITY.to_string()));
        assert!(manifest
            .modules
            .iter()
            .any(|module| module.kind == PluginModuleKind::Editor
                && module.crate_name == "zircon_plugin_sdk_examples_editor"));
        assert!(manifest
            .default_packaging
            .contains(&ExportPackagingStrategy::SourceTemplate));
    }
}
