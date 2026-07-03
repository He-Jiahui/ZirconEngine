use crate::plugin::PluginPackageManifest;

use super::super::RuntimePluginDescriptor;

pub(super) fn assign_descriptor_package_manifest_rows(
    descriptor: &RuntimePluginDescriptor,
    mut manifest: PluginPackageManifest,
) -> PluginPackageManifest {
    for capability in &descriptor.capabilities {
        manifest = manifest.with_capability(capability.clone());
    }
    for status in &descriptor.capability_statuses {
        manifest = manifest.with_capability_status(status.clone());
    }
    for interface in &descriptor.provided_interfaces {
        manifest = manifest.with_provided_interface(interface.clone());
    }
    for feature in &descriptor.optional_features {
        manifest = manifest.with_optional_feature(feature.clone());
    }
    manifest.default_packaging = descriptor.default_packaging.clone();
    manifest
}
