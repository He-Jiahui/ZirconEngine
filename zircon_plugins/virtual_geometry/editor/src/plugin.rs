use zircon_plugin_editor_support::{
    register_authoring_extensions, EditorAuthoringExtensions, EditorAuthoringSurface,
};

use crate::capability::{PLUGIN_ID, VIRTUAL_GEOMETRY_AUTHORING_CAPABILITY};
use crate::extension_ids::{
    VIRTUAL_GEOMETRY_AUTHORING_VIEW_ID, VIRTUAL_GEOMETRY_DRAWER_ID, VIRTUAL_GEOMETRY_TEMPLATE_ID,
};

#[derive(Clone, Debug)]
pub struct VirtualGeometryEditorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl VirtualGeometryEditorPlugin {
    pub fn new() -> Self {
        Self {
            descriptor: editor_plugin_descriptor(),
        }
    }
}

impl zircon_editor::EditorPlugin for VirtualGeometryEditorPlugin {
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
                drawer_id: VIRTUAL_GEOMETRY_DRAWER_ID,
                drawer_display_name: "Virtual Geometry Tools",
                template_id: VIRTUAL_GEOMETRY_TEMPLATE_ID,
                template_document: "plugins://virtual_geometry/editor/authoring.zui",
                surfaces: &[EditorAuthoringSurface::new(
                    VIRTUAL_GEOMETRY_AUTHORING_VIEW_ID,
                    "Virtual Geometry",
                    "Rendering",
                    "Plugins/Virtual Geometry",
                )],
            },
        )
    }
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(
        PLUGIN_ID,
        "Virtual Geometry",
        "zircon_plugin_virtual_geometry_editor",
    )
    .with_capability(VIRTUAL_GEOMETRY_AUTHORING_CAPABILITY)
}

pub fn editor_plugin() -> VirtualGeometryEditorPlugin {
    VirtualGeometryEditorPlugin::new()
}

pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    zircon_editor::EditorPlugin::package_manifest(
        &editor_plugin(),
        zircon_plugin_virtual_geometry_runtime::package_manifest(),
    )
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_plugin()).to_vec()
}

pub fn plugin_registration() -> zircon_editor::EditorPluginRegistrationReport {
    zircon_editor::EditorPluginRegistrationReport::from_plugin(
        &editor_plugin(),
        zircon_plugin_virtual_geometry_runtime::package_manifest(),
    )
}

pub fn editor_host_contract_marker() -> &'static str {
    zircon_editor::EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY
}
