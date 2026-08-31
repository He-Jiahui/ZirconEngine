use std::collections::HashSet;

use super::super::super::package_validation::RuntimePluginPackageValidationProjection;
use crate::plugin::{PluginPackageManifest, RuntimeExtensionRegistry};

pub(in crate::plugin::runtime_plugin::registration_report) fn validate_runtime_plugin_registration_interfaces(
    package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    extensions: &RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
) {
    let exported_interface_rows = extensions.plugin_interfaces();
    let (exported_interface_capacity, _) = exported_interface_rows.size_hint();
    let mut exported_interfaces = HashSet::with_capacity(exported_interface_capacity);
    for (owner, export) in exported_interface_rows {
        let Some(module_name) = extensions.plugin_module_name(owner) else {
            continue;
        };
        if projection.is_runtime_module(module_name) {
            exported_interfaces.insert(export.interface_id());
        }
    }

    let imported_interface_rows = extensions.plugin_interface_imports();
    let (imported_interface_capacity, _) = imported_interface_rows.size_hint();
    let mut imported_interfaces = HashSet::with_capacity(imported_interface_capacity);
    for (owner, import) in imported_interface_rows {
        let Some(module_name) = extensions.plugin_module_name(owner) else {
            continue;
        };
        if projection.is_runtime_module(module_name) {
            imported_interfaces.insert(import.interface_id());
        }
    }

    for interface_id in projection.provided_interface_ids() {
        if !exported_interfaces.contains(interface_id) {
            diagnostics.push(format!(
                "runtime plugin package `{}` declares interface `{}` but no runtime module exported it",
                package_manifest.id, interface_id
            ));
        }
    }

    for (owner, export) in extensions.plugin_interfaces() {
        let Some(module_name) = extensions.plugin_module_name(owner) else {
            continue;
        };
        if !projection.declares_provided_interface(export.interface_id()) {
            diagnostics.push(format!(
                "runtime plugin module `{module_name}` exported interface `{}` but package manifest did not declare it",
                export.interface_id()
            ));
        }
    }

    for interface_id in projection.dependency_interface_ids() {
        if !imported_interfaces.contains(interface_id) {
            diagnostics.push(format!(
                "runtime plugin package `{}` declares dependency interface `{interface_id}` but no runtime module imported it",
                package_manifest.id
            ));
        }
    }

    for (owner, import) in extensions.plugin_interface_imports() {
        let Some(module_name) = extensions.plugin_module_name(owner) else {
            continue;
        };
        if !projection.declares_dependency_interface(import.interface_id()) {
            diagnostics.push(format!(
                "runtime plugin module `{module_name}` imported interface `{}` but package dependencies did not declare it",
                import.interface_id()
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::bridge::PluginInterface;
    use crate::plugin::{
        PluginDependencyManifest, PluginModuleManifest, PluginPackageManifest,
        RuntimeExtensionRegistry,
    };

    use super::validate_runtime_plugin_registration_interfaces;
    use super::RuntimePluginPackageValidationProjection;

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

        let manifest = package_manifest(false);
        let projection = RuntimePluginPackageValidationProjection::build(&manifest);
        validate_runtime_plugin_registration_interfaces(
            &manifest,
            &projection,
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
        let manifest = package_manifest(true);
        let projection = RuntimePluginPackageValidationProjection::build(&manifest);
        validate_runtime_plugin_registration_interfaces(
            &manifest,
            &projection,
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
            &manifest,
            &projection,
            &registry,
            &mut diagnostics,
        );
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn preallocated_interface_sets_preserve_registration_validation_contract() {
        let source = include_str!("interfaces.rs");
        let preallocated_set = ["HashSet::with_", "capacity"].concat();
        let capacity_hint = [".size_", "hint()"].concat();
        let unbounded_collect = ["collect::<HashSet", "<_>>()"].concat();

        assert_eq!(source.matches(&preallocated_set).count(), 2);
        assert_eq!(source.matches(&capacity_hint).count(), 2);
        assert!(!source.contains(&unbounded_collect));
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
