mod registration;

use std::sync::{Arc, Mutex};

use zircon_editor::core::editor_extension::{
    EditorExtensionRegistry, EditorExtensionRegistryError,
};
use zircon_editor::core::runtime_event_consumer::EditorRuntimeEventConsumerRegistry;
use zircon_editor::{EditorPlugin, EditorPluginDescriptor, EditorPluginRegistrationReport};
use zircon_plugin_sdk::EditorPluginDeclaration;
use zircon_runtime::plugin::{PluginMaturity, PluginPackageManifest};

use self::registration::register_navigation_extensions;
use crate::capability::{EDITOR_CAPABILITIES, PLUGIN_ID};
use crate::runtime_mirror::{navigation_runtime_event_consumers_with_mirror, NavigationPieMirror};

#[derive(Clone, Debug)]
pub struct NavigationEditorPlugin {
    declaration: EditorPluginDeclaration,
    pie_mirror: Arc<Mutex<NavigationPieMirror>>,
}

impl Default for NavigationEditorPlugin {
    fn default() -> Self {
        let pie_mirror = Arc::new(Mutex::new(NavigationPieMirror::default()));
        let declaration = navigation_runtime_event_consumers_with_mirror(pie_mirror.clone())
            .into_iter()
            .fold(
                EditorPluginDeclaration::new(
                    PLUGIN_ID,
                    "Navigation",
                    "zircon_plugin_navigation_editor",
                )
                .with_category("runtime")
                .with_description(
                    "Navigation bake, viewport overlay, and PIE debugging authoring extensions.",
                )
                .with_maturity(PluginMaturity::Beta)
                .mirrors_runtime_manifest(zircon_plugin_navigation_runtime::package_manifest())
                .with_capabilities(EDITOR_CAPABILITIES.iter().copied()),
                |declaration, registration| {
                    declaration.with_runtime_event_consumer_registration(registration)
                },
            );
        Self {
            declaration,
            pie_mirror,
        }
    }
}

impl NavigationEditorPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn declaration(&self) -> &EditorPluginDeclaration {
        &self.declaration
    }

    pub fn pie_mirror(&self) -> Arc<Mutex<NavigationPieMirror>> {
        self.pie_mirror.clone()
    }

    pub fn package_manifest(&self) -> PluginPackageManifest {
        self.declaration.package_manifest()
    }

    pub fn editor_capabilities(&self) -> Vec<String> {
        self.declaration.capabilities().to_vec()
    }

    pub fn registration_report(&self) -> EditorPluginRegistrationReport {
        self.declaration.registration_report(self)
    }
}

impl EditorPlugin for NavigationEditorPlugin {
    fn descriptor(&self) -> &EditorPluginDescriptor {
        self.declaration.descriptor()
    }

    fn register_editor_extensions(
        &self,
        registry: &mut EditorExtensionRegistry,
    ) -> Result<(), EditorExtensionRegistryError> {
        register_navigation_extensions(registry, self.pie_mirror())
    }

    fn runtime_event_consumers(&self) -> EditorRuntimeEventConsumerRegistry {
        self.declaration.runtime_event_consumers()
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

pub fn package_manifest() -> PluginPackageManifest {
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
