use zircon_plugin_editor_support::{register_authoring_extensions, EditorAuthoringExtensions};

use crate::authoring;
use crate::capability::{NET_AUTHORING_CAPABILITY, PLUGIN_ID};
use crate::{NET_AUTHORING_SURFACES, NET_DRAWER_ID, NET_TEMPLATE_ID};

#[derive(Clone, Debug)]
pub struct NetEditorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl NetEditorPlugin {
    pub fn new() -> Self {
        Self {
            descriptor: editor_plugin_descriptor(),
        }
    }
}

impl zircon_editor::EditorPlugin for NetEditorPlugin {
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
                drawer_id: NET_DRAWER_ID,
                drawer_display_name: "Network Tools",
                template_id: NET_TEMPLATE_ID,
                template_document: "plugins://net/editor/authoring.zui",
                surfaces: NET_AUTHORING_SURFACES,
            },
        )?;
        authoring::register_net_authoring_workflows(registry)
    }
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(PLUGIN_ID, "Network", "zircon_plugin_net_editor")
        .with_capability(NET_AUTHORING_CAPABILITY)
}

pub fn editor_plugin() -> NetEditorPlugin {
    NetEditorPlugin::new()
}

pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    zircon_editor::EditorPlugin::package_manifest(
        &editor_plugin(),
        zircon_plugin_net_runtime::package_manifest(),
    )
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_plugin()).to_vec()
}

pub fn plugin_registration() -> zircon_editor::EditorPluginRegistrationReport {
    zircon_editor::EditorPluginRegistrationReport::from_plugin(
        &editor_plugin(),
        zircon_plugin_net_runtime::package_manifest(),
    )
}

pub fn editor_host_contract_marker() -> &'static str {
    zircon_editor::EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY
}
