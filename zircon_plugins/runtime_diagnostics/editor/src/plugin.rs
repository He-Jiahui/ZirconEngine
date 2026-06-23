use zircon_plugin_editor_support::{
    register_authoring_extensions, EditorAuthoringExtensions, EditorAuthoringSurface,
};
use zircon_runtime::builtin::RuntimeTargetMode;
use zircon_runtime::plugin::PluginPackageManifest;

use crate::{
    CAPABILITY, EDITOR_CAPABILITIES, PLUGIN_ID, RUNTIME_DIAGNOSTICS_DRAWER_ID,
    RUNTIME_DIAGNOSTICS_TEMPLATE_ID, RUNTIME_DIAGNOSTICS_VIEW_ID,
};

#[derive(Clone, Debug)]
pub struct RuntimeDiagnosticsEditorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl RuntimeDiagnosticsEditorPlugin {
    pub fn new() -> Self {
        Self {
            descriptor: editor_plugin_descriptor(),
        }
    }
}

impl zircon_editor::EditorPlugin for RuntimeDiagnosticsEditorPlugin {
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
                drawer_id: RUNTIME_DIAGNOSTICS_DRAWER_ID,
                drawer_display_name: "Runtime Diagnostics Tools",
                template_id: RUNTIME_DIAGNOSTICS_TEMPLATE_ID,
                template_document: "plugins://runtime_diagnostics/editor/authoring.zui",
                surfaces: &[EditorAuthoringSurface::new(
                    RUNTIME_DIAGNOSTICS_VIEW_ID,
                    "Runtime Diagnostics",
                    "Diagnostics",
                    "Plugins/Runtime Diagnostics",
                )],
            },
        )
    }
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(
        PLUGIN_ID,
        "Runtime Diagnostics",
        "zircon_plugin_runtime_diagnostics_editor",
    )
    .with_capability(CAPABILITY)
}

pub fn editor_plugin() -> RuntimeDiagnosticsEditorPlugin {
    RuntimeDiagnosticsEditorPlugin::new()
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
    PluginPackageManifest::new(PLUGIN_ID, "Runtime Diagnostics")
        .with_category("diagnostics")
        .with_supported_targets([RuntimeTargetMode::EditorHost])
        .with_capabilities(EDITOR_CAPABILITIES.iter().copied())
}
