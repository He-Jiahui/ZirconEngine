use zircon_plugin_editor_support::{
    register_authoring_extensions, EditorAuthoringExtensions, EditorAuthoringSurface,
};
use zircon_plugin_sdk::{authoring_plugin, EditorPluginDeclaration};

use crate::capability::{EDITOR_CAPABILITIES, PLUGIN_ID};
use crate::extension_ids::{PHYSICS_AUTHORING_VIEW_ID, PHYSICS_DRAWER_ID, PHYSICS_TEMPLATE_ID};

authoring_plugin! {
    pub struct PhysicsEditorPlugin {
        package_id: PLUGIN_ID,
        display_name: "Physics",
        crate_name: "zircon_plugin_physics_editor",
        category: "runtime",
        description: "Physics editor authoring extensions.",
        maturity: zircon_runtime::plugin::PluginMaturity::Experimental,
        mirrors_runtime_manifest: zircon_plugin_physics_runtime::package_manifest(),
        capabilities: EDITOR_CAPABILITIES,
        register_extensions: register_physics_authoring_extensions,
    }
}

pub fn editor_plugin_declaration() -> EditorPluginDeclaration {
    editor_plugin().declaration().clone()
}

fn register_physics_authoring_extensions(
    registry: &mut zircon_editor::core::editor_extension::EditorExtensionRegistry,
) -> Result<(), zircon_editor::core::editor_extension::EditorExtensionRegistryError> {
    register_authoring_extensions(
        registry,
        EditorAuthoringExtensions {
            drawer_id: PHYSICS_DRAWER_ID,
            drawer_display_name: "Physics Tools",
            template_id: PHYSICS_TEMPLATE_ID,
            template_document: "plugins://physics/editor/authoring.zui",
            surfaces: &[EditorAuthoringSurface::new(
                PHYSICS_AUTHORING_VIEW_ID,
                "Physics",
                "World",
                "Plugins/Physics",
            )],
        },
    )
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    editor_plugin_declaration().descriptor().clone()
}

pub fn editor_plugin() -> PhysicsEditorPlugin {
    PhysicsEditorPlugin::new()
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
