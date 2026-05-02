use zircon_runtime::{PluginModuleManifest, PluginPackageManifest};

use crate::core::editor_extension::{EditorExtensionRegistry, EditorExtensionRegistryError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorPluginDescriptor {
    pub package_id: String,
    pub display_name: String,
    pub crate_name: String,
    pub capabilities: Vec<String>,
}

impl EditorPluginDescriptor {
    pub fn new(
        package_id: impl Into<String>,
        display_name: impl Into<String>,
        crate_name: impl Into<String>,
    ) -> Self {
        Self {
            package_id: package_id.into(),
            display_name: display_name.into(),
            crate_name: crate_name.into(),
            capabilities: Vec::new(),
        }
    }

    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }

    pub fn attach_to_package(&self, manifest: PluginPackageManifest) -> PluginPackageManifest {
        manifest.with_editor_module(
            PluginModuleManifest::editor(
                format!("{}.editor", self.package_id),
                self.crate_name.clone(),
            )
            .with_capabilities(self.capabilities.iter().cloned()),
        )
    }

    pub fn builtin_catalog() -> Vec<Self> {
        [
            (
                "sound",
                "Sound",
                "zircon_plugin_sound_editor",
                "editor.extension.sound_authoring",
            ),
            (
                "texture",
                "Texture",
                "zircon_plugin_texture_editor",
                "editor.extension.texture_authoring",
            ),
            (
                "net",
                "Network",
                "zircon_plugin_net_editor",
                "editor.extension.net_authoring",
            ),
            (
                "navigation",
                "Navigation",
                "zircon_plugin_navigation_editor",
                "editor.extension.navigation_authoring",
            ),
            (
                "particles",
                "Particles",
                "zircon_plugin_particles_editor",
                "editor.extension.particles_authoring",
            ),
            (
                "virtual_geometry",
                "Virtual Geometry",
                "zircon_plugin_virtual_geometry_editor",
                "editor.extension.virtual_geometry_authoring",
            ),
            (
                "hybrid_gi",
                "Hybrid GI",
                "zircon_plugin_hybrid_gi_editor",
                "editor.extension.hybrid_gi_authoring",
            ),
            (
                "runtime_diagnostics",
                "Runtime Diagnostics",
                "zircon_plugin_runtime_diagnostics_editor",
                "editor.extension.runtime_diagnostics",
            ),
            (
                "ui_asset_authoring",
                "UI Asset Authoring",
                "zircon_plugin_ui_asset_authoring_editor",
                "editor.extension.ui_asset_authoring",
            ),
            (
                "native_window_hosting",
                "Native Window Hosting",
                "zircon_plugin_native_window_hosting_editor",
                "editor.extension.native_window_hosting",
            ),
        ]
        .into_iter()
        .map(|(id, name, crate_name, capability)| {
            Self::new(id, name, crate_name).with_capability(capability)
        })
        .collect()
    }
}

pub trait EditorPlugin {
    fn descriptor(&self) -> &EditorPluginDescriptor;

    fn package_manifest(&self, runtime_manifest: PluginPackageManifest) -> PluginPackageManifest {
        self.descriptor().attach_to_package(runtime_manifest)
    }

    fn editor_capabilities(&self) -> &[String] {
        &self.descriptor().capabilities
    }

    fn register_editor_extensions(
        &self,
        _registry: &mut EditorExtensionRegistry,
    ) -> Result<(), EditorExtensionRegistryError> {
        Ok(())
    }
}

impl EditorPlugin for EditorPluginDescriptor {
    fn descriptor(&self) -> &EditorPluginDescriptor {
        self
    }
}

#[derive(Clone, Debug)]
pub struct EditorPluginRegistrationReport {
    pub package_manifest: PluginPackageManifest,
    pub capabilities: Vec<String>,
    pub extensions: EditorExtensionRegistry,
    pub diagnostics: Vec<String>,
}

impl EditorPluginRegistrationReport {
    pub fn from_plugin(plugin: &dyn EditorPlugin, runtime_manifest: PluginPackageManifest) -> Self {
        let mut extensions = EditorExtensionRegistry::default();
        let mut diagnostics = Vec::new();
        if let Err(error) = plugin.register_editor_extensions(&mut extensions) {
            diagnostics.push(error.to_string());
        }
        Self {
            package_manifest: plugin.package_manifest(runtime_manifest),
            capabilities: plugin.editor_capabilities().to_vec(),
            extensions,
            diagnostics,
        }
    }

    pub fn is_success(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

#[derive(Clone, Debug, Default)]
pub struct EditorPluginCatalog {
    registrations: Vec<EditorPluginRegistrationReport>,
    diagnostics: Vec<String>,
}

impl EditorPluginCatalog {
    pub fn from_plugins<'a>(
        plugins: impl IntoIterator<Item = (&'a dyn EditorPlugin, PluginPackageManifest)>,
    ) -> Self {
        let mut catalog = Self::default();
        for (plugin, runtime_manifest) in plugins {
            catalog.register(plugin, runtime_manifest);
        }
        catalog
    }

    pub fn from_descriptors(
        descriptors: impl IntoIterator<Item = EditorPluginDescriptor>,
        runtime_manifests: impl IntoIterator<Item = PluginPackageManifest>,
    ) -> Self {
        let runtime_manifests = runtime_manifests.into_iter().collect::<Vec<_>>();
        let mut catalog = Self::default();
        for descriptor in descriptors {
            let runtime_manifest = runtime_manifests
                .iter()
                .find(|manifest| manifest.id == descriptor.package_id)
                .cloned()
                .unwrap_or_else(|| {
                    PluginPackageManifest::new(
                        descriptor.package_id.clone(),
                        descriptor.display_name.clone(),
                    )
                });
            catalog.register(&descriptor, runtime_manifest);
        }
        catalog
    }

    pub fn builtin(runtime_manifests: impl IntoIterator<Item = PluginPackageManifest>) -> Self {
        Self::from_descriptors(EditorPluginDescriptor::builtin_catalog(), runtime_manifests)
    }

    pub fn register(&mut self, plugin: &dyn EditorPlugin, runtime_manifest: PluginPackageManifest) {
        let report = EditorPluginRegistrationReport::from_plugin(plugin, runtime_manifest);
        self.diagnostics.extend(report.diagnostics.iter().cloned());
        self.registrations.push(report);
    }

    pub fn registrations(&self) -> &[EditorPluginRegistrationReport] {
        &self.registrations
    }

    pub fn package_manifests(&self) -> Vec<PluginPackageManifest> {
        self.registrations
            .iter()
            .map(|registration| registration.package_manifest.clone())
            .collect()
    }

    pub fn capabilities(&self) -> Vec<String> {
        let mut capabilities = self
            .registrations
            .iter()
            .flat_map(|registration| registration.capabilities.iter().cloned())
            .collect::<Vec<_>>();
        capabilities.sort();
        capabilities.dedup();
        capabilities
    }

    pub fn capabilities_for_package(&self, package_id: &str) -> Vec<String> {
        self.registrations
            .iter()
            .filter(|registration| registration.package_manifest.id == package_id)
            .flat_map(|registration| registration.capabilities.iter().cloned())
            .collect()
    }

    pub fn editor_extensions(&self) -> EditorExtensionCatalogReport {
        let mut registry = EditorExtensionRegistry::default();
        let mut diagnostics = Vec::new();
        for registration in &self.registrations {
            for view in registration.extensions.views() {
                push_editor_extension_result(
                    registry.register_view((*view).clone()),
                    &mut diagnostics,
                );
            }
            for drawer in registration.extensions.drawers() {
                push_editor_extension_result(
                    registry.register_drawer((*drawer).clone()),
                    &mut diagnostics,
                );
            }
            for menu_item in registration.extensions.menu_items() {
                push_editor_extension_result(
                    registry.register_menu_item((*menu_item).clone()),
                    &mut diagnostics,
                );
            }
            for component_drawer in registration.extensions.component_drawers() {
                push_editor_extension_result(
                    registry.register_component_drawer((*component_drawer).clone()),
                    &mut diagnostics,
                );
            }
            for ui_template in registration.extensions.ui_templates() {
                push_editor_extension_result(
                    registry.register_ui_template((*ui_template).clone()),
                    &mut diagnostics,
                );
            }
            for operation in registration.extensions.operations().descriptors().cloned() {
                push_editor_extension_result(
                    registry.register_operation(operation),
                    &mut diagnostics,
                );
            }
        }
        EditorExtensionCatalogReport {
            registry,
            diagnostics,
        }
    }

    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }

    pub fn is_success(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

#[derive(Clone, Debug)]
pub struct EditorExtensionCatalogReport {
    pub registry: EditorExtensionRegistry,
    pub diagnostics: Vec<String>,
}

impl EditorExtensionCatalogReport {
    pub fn is_success(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

fn push_editor_extension_result(
    result: Result<(), EditorExtensionRegistryError>,
    diagnostics: &mut Vec<String>,
) {
    if let Err(error) = result {
        diagnostics.push(error.to_string());
    }
}
