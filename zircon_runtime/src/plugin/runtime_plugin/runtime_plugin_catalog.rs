use crate::{
    PluginPackageManifest, ProjectPluginManifest, ProjectPluginSelection, RuntimeExtensionRegistry,
    RuntimeExtensionRegistryError, RuntimePlugin,
};

use super::RuntimePluginRegistrationReport;

#[derive(Clone, Debug, Default)]
pub struct RuntimePluginCatalog {
    registrations: Vec<RuntimePluginRegistrationReport>,
    diagnostics: Vec<String>,
}

impl RuntimePluginCatalog {
    pub fn from_plugins<'a>(plugins: impl IntoIterator<Item = &'a dyn RuntimePlugin>) -> Self {
        let mut catalog = Self::default();
        for plugin in plugins {
            catalog.register(plugin);
        }
        catalog
    }

    pub fn from_descriptors(
        descriptors: impl IntoIterator<Item = super::RuntimePluginDescriptor>,
    ) -> Self {
        let mut catalog = Self::default();
        for descriptor in descriptors {
            catalog.register(&descriptor);
        }
        catalog
    }

    pub fn builtin() -> Self {
        Self::from_descriptors(super::RuntimePluginDescriptor::builtin_catalog())
    }

    pub fn register(&mut self, plugin: &dyn RuntimePlugin) {
        let report = RuntimePluginRegistrationReport::from_plugin(plugin);
        self.diagnostics.extend(report.diagnostics.iter().cloned());
        self.registrations.push(report);
    }

    pub fn registrations(&self) -> &[RuntimePluginRegistrationReport] {
        &self.registrations
    }

    pub fn package_manifests(&self) -> Vec<PluginPackageManifest> {
        self.registrations
            .iter()
            .map(|registration| registration.package_manifest.clone())
            .collect()
    }

    pub fn project_manifest(&self) -> ProjectPluginManifest {
        ProjectPluginManifest {
            selections: self
                .registrations
                .iter()
                .map(|registration| registration.project_selection.clone())
                .collect(),
        }
    }

    pub fn complete_project_manifest(
        &self,
        manifest: &ProjectPluginManifest,
    ) -> ProjectPluginManifest {
        let mut completed = manifest.clone();
        for registration in &self.registrations {
            if completed
                .selections
                .iter()
                .any(|selection| selection.id == registration.project_selection.id)
            {
                continue;
            }
            let mut selection = registration.project_selection.clone();
            selection.enabled = false;
            completed.selections.push(selection);
        }
        for selection in &mut completed.selections {
            if let Some(catalog_selection) = self.project_selection_for_package(&selection.id) {
                if selection.runtime_crate.is_none() {
                    selection.runtime_crate = catalog_selection.runtime_crate.clone();
                }
                if selection.target_modes.is_empty() {
                    selection.target_modes = catalog_selection.target_modes.clone();
                }
            }
        }
        completed
    }

    pub fn project_selection_for_package(
        &self,
        package_id: &str,
    ) -> Option<ProjectPluginSelection> {
        self.registrations
            .iter()
            .find(|registration| registration.package_manifest.id == package_id)
            .map(|registration| registration.project_selection.clone())
    }

    pub fn runtime_extensions(&self) -> RuntimeExtensionCatalogReport {
        let mut registry = RuntimeExtensionRegistry::default();
        let mut diagnostics = Vec::new();
        for registration in &self.registrations {
            let plugin_id = registration.package_manifest.id.clone();
            for manager in registration.extensions.managers() {
                if let Err(error) = registry.register_manager(plugin_id.clone(), manager.clone()) {
                    diagnostics.push(error.to_string());
                }
            }
            for module in registration.extensions.modules() {
                push_runtime_extension_result(
                    registry.register_module(module.clone()),
                    &mut diagnostics,
                );
            }
            for render_feature in registration.extensions.render_features() {
                push_runtime_extension_result(
                    registry.register_render_feature(render_feature.clone()),
                    &mut diagnostics,
                );
            }
            for executor in registration.extensions.render_pass_executors() {
                push_runtime_extension_result(
                    registry.register_render_pass_executor(executor.clone()),
                    &mut diagnostics,
                );
            }
            for provider in registration.extensions.virtual_geometry_runtime_providers() {
                push_runtime_extension_result(
                    registry.register_virtual_geometry_runtime_provider(provider.clone()),
                    &mut diagnostics,
                );
            }
            for component in registration.extensions.components() {
                push_runtime_extension_result(
                    registry.register_component(component.clone()),
                    &mut diagnostics,
                );
            }
            for ui_component in registration.extensions.ui_components() {
                push_runtime_extension_result(
                    registry.register_ui_component(ui_component.clone()),
                    &mut diagnostics,
                );
            }
        }
        RuntimeExtensionCatalogReport {
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
pub struct RuntimeExtensionCatalogReport {
    pub registry: RuntimeExtensionRegistry,
    pub diagnostics: Vec<String>,
}

impl RuntimeExtensionCatalogReport {
    pub fn is_success(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

fn push_runtime_extension_result(
    result: Result<(), RuntimeExtensionRegistryError>,
    diagnostics: &mut Vec<String>,
) {
    if let Err(error) = result {
        diagnostics.push(error.to_string());
    }
}
