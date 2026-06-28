use zircon_plugin_editor_support::{register_authoring_extensions, EditorAuthoringExtensions};
use zircon_plugin_sdk::{authoring_plugin, EditorPluginDeclaration};

use crate::authoring;
use crate::capability::{EDITOR_CAPABILITIES, PLUGIN_ID};
use crate::{NET_AUTHORING_SURFACES, NET_DRAWER_ID, NET_TEMPLATE_ID};

authoring_plugin! {
    pub struct NetEditorPlugin {
        package_id: PLUGIN_ID,
        display_name: "Network",
        crate_name: "zircon_plugin_net_editor",
        category: "runtime",
        description: "Network editor authoring extensions.",
        maturity: zircon_runtime::plugin::PluginMaturity::Beta,
        mirrors_runtime_manifest: zircon_plugin_net_runtime::package_manifest(),
        capabilities: EDITOR_CAPABILITIES,
        register_extensions: register_net_editor_extensions,
    }
}

pub fn editor_plugin_declaration() -> EditorPluginDeclaration {
    editor_plugin().declaration().clone()
}

fn register_net_editor_extensions(
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

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    editor_plugin_declaration().descriptor().clone()
}

pub fn editor_plugin() -> NetEditorPlugin {
    NetEditorPlugin::new()
}

pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    editor_plugin().declaration().package_manifest()
}

pub fn editor_capabilities() -> Vec<String> {
    editor_plugin().declaration().capabilities().to_vec()
}

pub fn plugin_registration() -> zircon_editor::EditorPluginRegistrationReport {
    let plugin = editor_plugin();
    plugin.declaration().registration_report(&plugin)
}

pub fn editor_host_contract_marker() -> &'static str {
    zircon_editor::EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY
}
