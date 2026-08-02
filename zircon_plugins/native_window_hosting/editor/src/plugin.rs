use zircon_plugin_editor_support::{
    register_authoring_extensions, EditorAuthoringExtensions, EditorAuthoringSurface,
};
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::plugin::{
    PluginDistributionManifest, PluginModuleManifest, PluginPackageManifest,
};

use crate::{
    CAPABILITY, EDITOR_CRATE_NAME, NATIVE_EDITOR_ENTRY, NATIVE_WINDOW_DRAWER_ID,
    NATIVE_WINDOW_HOSTING_DECLARATION, NATIVE_WINDOW_TEMPLATE_ID, PREFAB_WINDOW_VIEW_ID,
    WORKBENCH_WINDOW_VIEW_ID,
};

pub const NATIVE_WINDOW_HOSTING_DIST_CRATE_NAME: &str = "zircon_plugin_native_window_hosting_dist";
pub const NATIVE_WINDOW_HOSTING_DIST_EDITOR_ENTRY: &str = NATIVE_EDITOR_ENTRY.name();
const NATIVE_WINDOW_HOSTING_DIST_ENGINE_COMPAT: &str = ">=0.1, <0.2";
const NATIVE_DESCRIPTOR_SYMBOL_V3: &str = "zircon_native_plugin_descriptor_v3";
const NATIVE_ABI_VERSION_V3: u32 = 3;

#[derive(Clone, Debug)]
pub struct NativeWindowHostingEditorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl NativeWindowHostingEditorPlugin {
    pub fn new() -> Self {
        Self {
            descriptor: editor_plugin_descriptor(),
        }
    }
}

impl zircon_editor::EditorPlugin for NativeWindowHostingEditorPlugin {
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
                drawer_id: NATIVE_WINDOW_DRAWER_ID,
                drawer_display_name: "Native Window Tools",
                template_id: NATIVE_WINDOW_TEMPLATE_ID,
                template_document: "plugins://native_window_hosting/editor/authoring.zui",
                surfaces: &[
                    EditorAuthoringSurface::new(
                        WORKBENCH_WINDOW_VIEW_ID,
                        "Workbench",
                        "Window",
                        "Plugins/Native Windows/Workbench",
                    ),
                    EditorAuthoringSurface::new(
                        PREFAB_WINDOW_VIEW_ID,
                        "Prefab Editor",
                        "Window",
                        "Plugins/Native Windows/Prefab",
                    ),
                ],
            },
        )
    }
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(
        NATIVE_WINDOW_HOSTING_DECLARATION.id(),
        NATIVE_WINDOW_HOSTING_DECLARATION.display_name(),
        EDITOR_CRATE_NAME,
    )
    .with_capability(CAPABILITY)
}

pub fn editor_plugin() -> NativeWindowHostingEditorPlugin {
    NativeWindowHostingEditorPlugin::new()
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
    PluginPackageManifest::new(
        NATIVE_WINDOW_HOSTING_DECLARATION.id(),
        NATIVE_WINDOW_HOSTING_DECLARATION.display_name(),
    )
    .with_category(NATIVE_WINDOW_HOSTING_DECLARATION.category())
    .with_supported_targets(NATIVE_WINDOW_HOSTING_DECLARATION.target_modes())
    .with_supported_platforms(NATIVE_WINDOW_HOSTING_DECLARATION.supported_platforms())
    .with_capabilities(
        NATIVE_WINDOW_HOSTING_DECLARATION
            .capabilities()
            .iter()
            .copied(),
    )
    .with_default_packaging(NATIVE_WINDOW_HOSTING_DECLARATION.default_packaging())
    .with_native_module(native_window_hosting_dist_module_manifest())
    .with_distribution(PluginDistributionManifest {
        forms: vec!["dist".to_string()],
        default_packaging: vec![ExportPackagingStrategy::NativeDynamic],
        abi_version: Some(NATIVE_ABI_VERSION_V3),
        engine_compat: NATIVE_WINDOW_HOSTING_DIST_ENGINE_COMPAT.to_string(),
        dist_crate: NATIVE_WINDOW_HOSTING_DIST_CRATE_NAME.to_string(),
        descriptor_symbol: NATIVE_DESCRIPTOR_SYMBOL_V3.to_string(),
        editor_entry: NATIVE_WINDOW_HOSTING_DIST_EDITOR_ENTRY.to_string(),
        ..PluginDistributionManifest::default()
    })
}

pub fn native_window_hosting_dist_module_manifest() -> PluginModuleManifest {
    PluginModuleManifest::native(
        "native_window_hosting.dist",
        NATIVE_WINDOW_HOSTING_DIST_CRATE_NAME,
    )
    .with_target_modes([RuntimeTargetMode::EditorHost])
    .with_capabilities(
        NATIVE_WINDOW_HOSTING_DECLARATION
            .capabilities()
            .iter()
            .copied(),
    )
}
