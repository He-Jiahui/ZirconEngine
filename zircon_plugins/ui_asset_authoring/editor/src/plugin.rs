use zircon_editor::core::asset::{
    AssetContextCommandDescriptor, AssetCreationTemplateDescriptor, AssetToolkitDescriptor,
    AssetTypeContribution, AssetTypeId,
};
use zircon_editor::core::commands::EditorCommandDescriptor;
use zircon_editor::core::editor_event::{EditorEvent, MenuAction, ViewDescriptorId};
use zircon_editor::core::editor_operation::EditorOperationPath;
use zircon_plugin_editor_support::{
    register_authoring_extensions, EditorAuthoringExtensions, EditorAuthoringSurface,
};
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::{
    core::framework::project::ExportPackagingStrategy,
    core::framework::project::ExportTargetPlatform, plugin::PluginDistributionManifest,
    plugin::PluginModuleManifest, plugin::PluginPackageManifest,
};
use zircon_runtime_interface::resource::ResourceKind;

use crate::{
    CAPABILITY, EDITOR_CAPABILITIES, PLUGIN_ID, UI_ASSET_DRAWER_ID, UI_ASSET_TEMPLATE_ID,
    UI_ASSET_VIEW_ID,
};

pub const UI_ASSET_AUTHORING_DIST_CRATE_NAME: &str = "zircon_plugin_ui_asset_authoring_dist";
pub const UI_ASSET_AUTHORING_DIST_EDITOR_ENTRY: &str =
    "zircon_plugin_ui_asset_authoring_editor_entry_v3";
const UI_ASSET_AUTHORING_DIST_ENGINE_COMPAT: &str = ">=0.1, <0.2";
const NATIVE_DESCRIPTOR_SYMBOL_V3: &str = "zircon_native_plugin_descriptor_v3";
const NATIVE_ABI_VERSION_V3: u32 = 3;

#[derive(Clone, Debug)]
pub struct UiAssetAuthoringEditorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl UiAssetAuthoringEditorPlugin {
    pub fn new() -> Self {
        Self {
            descriptor: editor_plugin_descriptor(),
        }
    }
}

impl zircon_editor::EditorPlugin for UiAssetAuthoringEditorPlugin {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }

    fn register_editor_extensions(
        &self,
        registry: &mut zircon_editor::core::editor_extension::EditorExtensionRegistry,
    ) -> Result<(), zircon_editor::core::editor_extension::EditorExtensionRegistryError> {
        register_authoring_extensions(
            registry,
            EditorAuthoringExtensions {
                drawer_id: UI_ASSET_DRAWER_ID,
                drawer_display_name: "UI Asset Tools",
                template_id: UI_ASSET_TEMPLATE_ID,
                template_document: "plugins://ui_asset_authoring/editor/authoring.zui",
                surfaces: &[EditorAuthoringSurface::new(
                    UI_ASSET_VIEW_ID,
                    "UI Asset",
                    "Assets",
                    "Plugins/UI Asset",
                )],
            },
        )?;
        let open = EditorOperationPath::parse("view.editor.ui_asset.open").map_err(
            zircon_editor::core::editor_extension::EditorExtensionRegistryError::OperationPath,
        )?;
        for (kind, template_id, create_path, display_name, document) in [
            (
                ResourceKind::UiLayout,
                "ui_asset.layout",
                "ui_asset.create.layout",
                "UI Layout",
                "plugins://ui_asset_authoring/editor/templates/layout.zui",
            ),
            (
                ResourceKind::UiWidget,
                "ui_asset.widget",
                "ui_asset.create.widget",
                "UI Widget",
                "plugins://ui_asset_authoring/editor/templates/widget.zui",
            ),
            (
                ResourceKind::UiStyle,
                "ui_asset.style",
                "ui_asset.create.style",
                "UI Style",
                "plugins://ui_asset_authoring/editor/templates/style.zui",
            ),
        ] {
            let create = EditorOperationPath::parse(create_path).map_err(
                zircon_editor::core::editor_extension::EditorExtensionRegistryError::OperationPath,
            )?;
            registry.register_command(
                EditorCommandDescriptor::pending_operation(
                    create.clone(),
                    format!("Create {display_name}"),
                )
                .with_required_capabilities([CAPABILITY])
                .with_event(EditorEvent::WorkbenchMenu(MenuAction::OpenView(
                    ViewDescriptorId::new(UI_ASSET_VIEW_ID),
                ))),
            )?;
            registry.register_asset_type_contribution(
                AssetTypeContribution::augment(AssetTypeId::from_resource_kind(kind))
                    .with_toolkit(
                        AssetToolkitDescriptor::new(UI_ASSET_VIEW_ID, open.clone())
                            .with_required_capabilities([CAPABILITY]),
                    )
                    .with_creation_template(
                        AssetCreationTemplateDescriptor::new(template_id, display_name, create)
                            .with_default_document(document)
                            .with_required_capabilities([CAPABILITY]),
                    )
                    .with_context_command(
                        AssetContextCommandDescriptor::new(
                            format!("{template_id}.open"),
                            format!("Open {display_name}"),
                            open.clone(),
                        )
                        .with_icon_name("edit")
                        .with_required_capabilities([CAPABILITY]),
                    ),
            )?;
        }
        Ok(())
    }
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(
        PLUGIN_ID,
        "UI Asset Authoring",
        "zircon_plugin_ui_asset_authoring_editor",
    )
    .with_capability(CAPABILITY)
}

pub fn editor_plugin() -> UiAssetAuthoringEditorPlugin {
    UiAssetAuthoringEditorPlugin::new()
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
    PluginPackageManifest::new(PLUGIN_ID, "UI Asset Authoring")
        .with_category("authoring")
        .with_supported_targets([RuntimeTargetMode::EditorHost])
        .with_supported_platforms([
            ExportTargetPlatform::Windows,
            ExportTargetPlatform::Linux,
            ExportTargetPlatform::Macos,
        ])
        .with_capabilities(EDITOR_CAPABILITIES.iter().copied())
        .with_default_packaging([
            ExportPackagingStrategy::SourceTemplate,
            ExportPackagingStrategy::LibraryEmbed,
            ExportPackagingStrategy::NativeDynamic,
        ])
        .with_native_module(ui_asset_authoring_dist_module_manifest())
        .with_distribution(PluginDistributionManifest {
            forms: vec!["dist".to_string()],
            default_packaging: vec![ExportPackagingStrategy::NativeDynamic],
            abi_version: Some(NATIVE_ABI_VERSION_V3),
            engine_compat: UI_ASSET_AUTHORING_DIST_ENGINE_COMPAT.to_string(),
            dist_crate: UI_ASSET_AUTHORING_DIST_CRATE_NAME.to_string(),
            descriptor_symbol: NATIVE_DESCRIPTOR_SYMBOL_V3.to_string(),
            editor_entry: UI_ASSET_AUTHORING_DIST_EDITOR_ENTRY.to_string(),
            ..PluginDistributionManifest::default()
        })
}

pub fn ui_asset_authoring_dist_module_manifest() -> PluginModuleManifest {
    PluginModuleManifest::native(
        "ui_asset_authoring.dist",
        UI_ASSET_AUTHORING_DIST_CRATE_NAME,
    )
    .with_target_modes([RuntimeTargetMode::EditorHost])
    .with_capabilities(EDITOR_CAPABILITIES.iter().copied())
}
