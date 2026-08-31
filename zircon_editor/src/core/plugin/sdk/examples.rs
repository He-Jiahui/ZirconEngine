//! SDK fixture declarations for editor-plugin contributions.

use std::collections::BTreeMap;

use crate::core::asset::{AssetToolkitDescriptor, AssetTypeContribution, AssetTypeId};
use crate::core::commands::{
    EditorCommandDescriptor, EditorCommandMenuPath, EditorCommandPresentation,
};
use crate::core::editor_extension::{
    AssetImporterDescriptor, EditorExtensionRegistry, EditorExtensionRegistryError,
    EditorMenuItemDescriptor, EditorUiTemplateDescriptor, ViewDescriptor,
};
use crate::core::editor_operation::EditorOperationPath;
use crate::core::extension::InspectorCustomizationDescriptor;
use crate::core::i18n::EditorLocalizationBundle;
use crate::core::plugin::{EditorPlugin, EditorPluginDescriptor};
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
        registry.register_localization_bundle(example_window_bundle()?)?;
        let operation_path = parse_operation("sdk.example.toggle_weather_window")?;
        registry.register_command(
            EditorCommandDescriptor::localized_operation(
                operation_path.clone(),
                EditorCommandPresentation::localized(
                    "sdk_example_window",
                    "command.sdk.example.toggle_weather_window.label",
                    "command.sdk.example.toggle_weather_window.description",
                )
                .map_err(EditorExtensionRegistryError::View)?,
            )
            .with_menu_path(EditorCommandMenuPath::builtin(
                &operation_path,
                "tools",
                &["sdk_examples"],
            )),
        )?;
        registry.register_view(ViewDescriptor::new(
            "sdk.example.weather_window",
            "SDK Weather",
            "SDK Examples",
        ))?;
        registry.register_menu_item(EditorMenuItemDescriptor::for_operation(operation_path))
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
        registry.register_localization_bundle(example_asset_bundle()?)?;
        let import_operation = parse_operation("sdk.example.import_model")?;
        let open_operation = parse_operation("sdk.example.open_model_inspector")?;
        registry.register_command(
            EditorCommandDescriptor::localized_operation(
                import_operation.clone(),
                EditorCommandPresentation::localized(
                    "sdk_example_asset",
                    "command.sdk.example.import_model.label",
                    "command.sdk.example.import_model.description",
                )
                .map_err(EditorExtensionRegistryError::View)?,
            )
            .with_menu_path(EditorCommandMenuPath::builtin(
                &import_operation,
                "assets",
                &["sdk_examples"],
            )),
        )?;
        registry.register_command(EditorCommandDescriptor::localized_operation(
            open_operation.clone(),
            EditorCommandPresentation::localized(
                "sdk_example_asset",
                "command.sdk.example.open_model_inspector.label",
                "command.sdk.example.open_model_inspector.description",
            )
            .map_err(EditorExtensionRegistryError::View)?,
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
        registry.register_inspector_customization(InspectorCustomizationDescriptor::new(
            "sdk.example.ModelImportSettings",
            "asset://sdk_examples/editor/model_import_settings.zui",
            "sdk.example.ModelImportSettingsController",
        ))
    }
}

fn example_window_bundle() -> Result<EditorLocalizationBundle, EditorExtensionRegistryError> {
    EditorLocalizationBundle::from_locale_maps(
        "sdk_example_window",
        BTreeMap::from([
            (
                "en".to_string(),
                BTreeMap::from([
                    ("menu.tools.label".to_string(), "Tools".to_string()),
                    (
                        "menu.tools.sdk_examples.label".to_string(),
                        "SDK Examples".to_string(),
                    ),
                    (
                        "command.sdk.example.toggle_weather_window.label".to_string(),
                        "Toggle SDK Weather Window".to_string(),
                    ),
                    (
                        "command.sdk.example.toggle_weather_window.description".to_string(),
                        "Toggle the SDK weather window.".to_string(),
                    ),
                ]),
            ),
            (
                "zh-CN".to_string(),
                BTreeMap::from([
                    ("menu.tools.label".to_string(), "工具".to_string()),
                    (
                        "menu.tools.sdk_examples.label".to_string(),
                        "SDK 示例".to_string(),
                    ),
                    (
                        "command.sdk.example.toggle_weather_window.label".to_string(),
                        "切换 SDK 天气窗口".to_string(),
                    ),
                    (
                        "command.sdk.example.toggle_weather_window.description".to_string(),
                        "切换 SDK 天气窗口。".to_string(),
                    ),
                ]),
            ),
        ]),
    )
    .map_err(EditorExtensionRegistryError::View)
}

fn example_asset_bundle() -> Result<EditorLocalizationBundle, EditorExtensionRegistryError> {
    EditorLocalizationBundle::from_locale_maps(
        "sdk_example_asset",
        BTreeMap::from([
            (
                "en".to_string(),
                BTreeMap::from([
                    ("menu.assets.label".to_string(), "Assets".to_string()),
                    (
                        "menu.assets.sdk_examples.label".to_string(),
                        "SDK Examples".to_string(),
                    ),
                    (
                        "command.sdk.example.import_model.label".to_string(),
                        "Import SDK Model".to_string(),
                    ),
                    (
                        "command.sdk.example.import_model.description".to_string(),
                        "Import a model through the SDK example.".to_string(),
                    ),
                    (
                        "command.sdk.example.open_model_inspector.label".to_string(),
                        "Open SDK Model Inspector".to_string(),
                    ),
                    (
                        "command.sdk.example.open_model_inspector.description".to_string(),
                        "Open the SDK model inspector.".to_string(),
                    ),
                ]),
            ),
            (
                "zh-CN".to_string(),
                BTreeMap::from([
                    ("menu.assets.label".to_string(), "资产".to_string()),
                    (
                        "menu.assets.sdk_examples.label".to_string(),
                        "SDK 示例".to_string(),
                    ),
                    (
                        "command.sdk.example.import_model.label".to_string(),
                        "导入 SDK 模型".to_string(),
                    ),
                    (
                        "command.sdk.example.import_model.description".to_string(),
                        "通过 SDK 示例导入模型。".to_string(),
                    ),
                    (
                        "command.sdk.example.open_model_inspector.label".to_string(),
                        "打开 SDK 模型检查器".to_string(),
                    ),
                    (
                        "command.sdk.example.open_model_inspector.description".to_string(),
                        "打开 SDK 模型检查器。".to_string(),
                    ),
                ]),
            ),
        ]),
    )
    .map_err(EditorExtensionRegistryError::View)
}

fn parse_operation(path: &str) -> Result<EditorOperationPath, EditorExtensionRegistryError> {
    EditorOperationPath::parse(path).map_err(EditorExtensionRegistryError::OperationPath)
}
