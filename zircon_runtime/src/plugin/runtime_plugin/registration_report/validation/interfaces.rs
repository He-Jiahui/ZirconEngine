use crate::plugin::{PluginModuleKind, PluginPackageManifest, RuntimeExtensionRegistry};

pub(in crate::plugin::runtime_plugin::registration_report) fn validate_runtime_plugin_registration_interfaces(
    package_manifest: &PluginPackageManifest,
    extensions: &RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
) {
    let runtime_modules = package_manifest
        .modules
        .iter()
        .filter(|module| module.kind == PluginModuleKind::Runtime)
        .map(|module| module.name.as_str())
        .collect::<Vec<_>>();

    for declared in &package_manifest.provides_interfaces {
        if !package_exported_interface(extensions, &runtime_modules, &declared.id) {
            diagnostics.push(format!(
                "runtime plugin package `{}` declares interface `{}` but no runtime module exported it",
                package_manifest.id, declared.id
            ));
        }
    }

    for (owner, export) in extensions.plugin_interfaces() {
        let Some(module_name) = extensions.plugin_module_name(owner) else {
            continue;
        };
        if !package_manifest
            .provides_interfaces
            .iter()
            .any(|declared| declared.id == export.interface_id())
        {
            diagnostics.push(format!(
                "runtime plugin module `{module_name}` exported interface `{}` but package manifest did not declare it",
                export.interface_id()
            ));
        }
    }

    for dependency in &package_manifest.dependencies {
        for interface_id in &dependency.interfaces {
            if !package_imported_interface(extensions, &runtime_modules, interface_id) {
                diagnostics.push(format!(
                    "runtime plugin package `{}` declares dependency interface `{interface_id}` but no runtime module imported it",
                    package_manifest.id
                ));
            }
        }
    }

    for (owner, import) in extensions.plugin_interface_imports() {
        let Some(module_name) = extensions.plugin_module_name(owner) else {
            continue;
        };
        if !package_manifest.dependencies.iter().any(|dependency| {
            dependency
                .interfaces
                .iter()
                .any(|interface_id| interface_id == import.interface_id())
        }) {
            diagnostics.push(format!(
                "runtime plugin module `{module_name}` imported interface `{}` but package dependencies did not declare it",
                import.interface_id()
            ));
        }
    }
}

fn package_exported_interface(
    extensions: &RuntimeExtensionRegistry,
    runtime_modules: &[&str],
    interface_id: &str,
) -> bool {
    extensions.plugin_interfaces().any(|(owner, export)| {
        export.interface_id() == interface_id
            && extensions
                .plugin_module_name(owner)
                .is_some_and(|module_name| runtime_modules.contains(&module_name))
    })
}

fn package_imported_interface(
    extensions: &RuntimeExtensionRegistry,
    runtime_modules: &[&str],
    interface_id: &str,
) -> bool {
    extensions
        .plugin_interface_imports()
        .any(|(owner, import)| {
            import.interface_id() == interface_id
                && extensions
                    .plugin_module_name(owner)
                    .is_some_and(|module_name| runtime_modules.contains(&module_name))
        })
}

#[cfg(test)]
mod tests {
    use crate::core::framework::bridge::PluginInterface;
    use crate::plugin::{
        PluginDependencyManifest, PluginModuleManifest, PluginPackageManifest,
        RuntimeExtensionRegistry,
    };

    use super::validate_runtime_plugin_registration_interfaces;

    trait ImportedContract: Send + Sync {}

    impl PluginInterface for dyn ImportedContract {
        const INTERFACE_ID: &'static str = "test.imported.contract.v1";
    }

    #[test]
    fn undeclared_interface_import_is_rejected() {
        let mut registry = RuntimeExtensionRegistry::default();
        let owner = registry.intern_plugin_module("consumer.runtime").unwrap();
        registry
            .import_interface::<dyn ImportedContract>(owner)
            .unwrap();
        let mut diagnostics = Vec::new();

        validate_runtime_plugin_registration_interfaces(
            &package_manifest(false),
            &registry,
            &mut diagnostics,
        );

        assert!(diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("package dependencies did not declare")));
    }

    #[test]
    fn declared_interface_import_must_be_registered() {
        let registry = RuntimeExtensionRegistry::default();
        let mut diagnostics = Vec::new();
        validate_runtime_plugin_registration_interfaces(
            &package_manifest(true),
            &registry,
            &mut diagnostics,
        );
        assert!(diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("no runtime module imported it")));

        let mut registry = RuntimeExtensionRegistry::default();
        let owner = registry.intern_plugin_module("consumer.runtime").unwrap();
        registry
            .import_interface::<dyn ImportedContract>(owner)
            .unwrap();
        diagnostics.clear();
        validate_runtime_plugin_registration_interfaces(
            &package_manifest(true),
            &registry,
            &mut diagnostics,
        );
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    fn package_manifest(declare_import: bool) -> PluginPackageManifest {
        let manifest = PluginPackageManifest::new("consumer", "Consumer").with_runtime_module(
            PluginModuleManifest::runtime("consumer.runtime", "consumer_runtime"),
        );
        if declare_import {
            manifest.with_dependency(
                PluginDependencyManifest::new("provider", false)
                    .with_interface(<dyn ImportedContract as PluginInterface>::INTERFACE_ID),
            )
        } else {
            manifest
        }
    }
}
