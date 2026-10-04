use crate::core::framework::bridge::PluginInterface;
use crate::plugin::{
    PluginDependencyManifest, PluginModuleManifest, PluginPackageManifest, RuntimeExtensionRegistry,
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
    let source = include_str!("../interfaces.rs");
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
