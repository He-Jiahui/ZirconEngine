mod registration;

use zircon_plugin_sdk::{authoring_plugin, EditorPluginDeclaration};

use self::registration::register_navigation_extensions;
use crate::capability::{EDITOR_CAPABILITIES, PLUGIN_ID};
use crate::runtime_mirror::{
    navigation_runtime_event_consumers, NavigationPieMirror, NAVIGATION_TICK_CONSUMER_ID,
};

authoring_plugin! {
    pub struct NavigationEditorPlugin {
        package_id: PLUGIN_ID,
        display_name: "Navigation",
        crate_name: "zircon_plugin_navigation_editor",
        category: "runtime",
        description: "Navigation bake, viewport overlay, and PIE debugging authoring extensions.",
        maturity: zircon_runtime::plugin::PluginMaturity::Beta,
        mirrors_runtime_manifest: zircon_plugin_navigation_runtime::package_manifest(),
        capabilities: EDITOR_CAPABILITIES,
        runtime_event_consumers: navigation_runtime_event_consumers(),
        register_extensions: register_navigation_extensions,
    }
}

pub fn editor_plugin_declaration() -> EditorPluginDeclaration {
    editor_plugin().declaration().clone()
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    editor_plugin_declaration().descriptor().clone()
}

pub fn editor_plugin() -> NavigationEditorPlugin {
    NavigationEditorPlugin::new()
}

impl NavigationEditorPlugin {
    pub fn pie_mirror(&self) -> std::sync::Arc<std::sync::Mutex<NavigationPieMirror>> {
        self.declaration()
            .runtime_event_consumers()
            .registration(NAVIGATION_TICK_CONSUMER_ID)
            .and_then(|registration| registration.state::<NavigationPieMirror>())
            .expect("navigation PIE mirror declaration is registered")
    }
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
