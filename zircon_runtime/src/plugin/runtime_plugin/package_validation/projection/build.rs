use std::cell::Cell;
use std::collections::HashSet;

use crate::plugin::{PluginModuleKind, PluginPackageManifest};

use super::duplicate_identity::DuplicateIdentity;
use super::duplicate_occurrence::DuplicateOccurrence;
use super::RuntimePluginPackageValidationProjection;

mod contributions;
mod embedded_features;
mod interfaces;
mod modules;

use self::contributions::index_contribution_identities;
use self::embedded_features::index_embedded_features;
use self::interfaces::index_interfaces;
use self::modules::index_package_modules;

impl<'a> RuntimePluginPackageValidationProjection<'a> {
    pub(in crate::plugin::runtime_plugin) fn build(
        package_manifest: &'a PluginPackageManifest,
    ) -> Self {
        #[cfg(test)]
        super::metrics::observe_package_projection_build();

        let identity_row_capacity = package_identity_row_capacity(package_manifest);
        let mut seen = HashSet::with_capacity(identity_row_capacity);
        let mut duplicates = HashSet::new();
        let mut identity_rows_indexed = 0;

        for (index, capability) in package_manifest.capabilities.iter().enumerate() {
            index_identity(
                &mut seen,
                &mut duplicates,
                DuplicateIdentity::PackageCapability(capability),
                DuplicateOccurrence::PackageCapability(index),
                &mut identity_rows_indexed,
            );
        }
        for (index, root) in package_manifest.asset_roots.iter().enumerate() {
            index_identity(
                &mut seen,
                &mut duplicates,
                DuplicateIdentity::AssetRoot(root),
                DuplicateOccurrence::AssetRoot(index),
                &mut identity_rows_indexed,
            );
        }
        for (index, root) in package_manifest.content_roots.iter().enumerate() {
            index_identity(
                &mut seen,
                &mut duplicates,
                DuplicateIdentity::ContentRoot(root),
                DuplicateOccurrence::ContentRoot(index),
                &mut identity_rows_indexed,
            );
        }
        for (importer_index, importer) in package_manifest.asset_importers.iter().enumerate() {
            index_identity(
                &mut seen,
                &mut duplicates,
                DuplicateIdentity::AssetImporterId(&importer.id),
                DuplicateOccurrence::AssetImporterId(importer_index),
                &mut identity_rows_indexed,
            );
            for (capability_index, capability) in importer.required_capabilities.iter().enumerate()
            {
                index_identity(
                    &mut seen,
                    &mut duplicates,
                    DuplicateIdentity::AssetImporterCapability {
                        importer: importer_index,
                        value: capability,
                    },
                    DuplicateOccurrence::AssetImporterCapability {
                        importer: importer_index,
                        capability: capability_index,
                    },
                    &mut identity_rows_indexed,
                );
            }
        }
        for (dependency_index, dependency) in package_manifest.dependencies.iter().enumerate() {
            if let Some(capability) = dependency.capability.as_deref() {
                index_identity(
                    &mut seen,
                    &mut duplicates,
                    DuplicateIdentity::DependencyCapability {
                        provider: &dependency.id,
                        capability,
                    },
                    DuplicateOccurrence::DependencyCapability(dependency_index),
                    &mut identity_rows_indexed,
                );
            }
            for (interface_index, interface_id) in dependency.interfaces.iter().enumerate() {
                index_identity(
                    &mut seen,
                    &mut duplicates,
                    DuplicateIdentity::DependencyInterface {
                        dependency: dependency_index,
                        value: interface_id,
                    },
                    DuplicateOccurrence::DependencyInterface {
                        dependency: dependency_index,
                        interface: interface_index,
                    },
                    &mut identity_rows_indexed,
                );
            }
        }
        for (status_index, status) in package_manifest.capability_statuses.iter().enumerate() {
            index_identity(
                &mut seen,
                &mut duplicates,
                DuplicateIdentity::CapabilityStatus(&status.capability),
                DuplicateOccurrence::CapabilityStatus(status_index),
                &mut identity_rows_indexed,
            );
            for (reference_index, reference) in status.bevy_references.iter().enumerate() {
                index_identity(
                    &mut seen,
                    &mut duplicates,
                    DuplicateIdentity::CapabilityStatusReference {
                        status: status_index,
                        value: reference,
                    },
                    DuplicateOccurrence::CapabilityStatusReference {
                        status: status_index,
                        reference: reference_index,
                    },
                    &mut identity_rows_indexed,
                );
            }
        }

        index_contribution_identities(
            package_manifest,
            &mut seen,
            &mut duplicates,
            &mut identity_rows_indexed,
        );
        index_embedded_features(
            package_manifest,
            &mut seen,
            &mut duplicates,
            &mut identity_rows_indexed,
        );
        index_interfaces(
            package_manifest,
            &mut seen,
            &mut duplicates,
            &mut identity_rows_indexed,
        );
        index_package_modules(
            package_manifest,
            &mut seen,
            &mut duplicates,
            &mut identity_rows_indexed,
        );
        debug_assert_eq!(identity_rows_indexed, identity_row_capacity);

        let owned_capabilities = package_manifest
            .capabilities
            .iter()
            .chain(
                package_manifest
                    .optional_features
                    .iter()
                    .flat_map(|feature| feature.capabilities.iter()),
            )
            .map(String::as_str)
            .collect();
        let runtime_module_names = package_manifest
            .modules
            .iter()
            .filter(|module| module.kind == PluginModuleKind::Runtime)
            .map(|module| module.name.as_str())
            .collect::<Vec<_>>();
        let runtime_module_name_membership = runtime_module_names.iter().copied().collect();
        let provided_interface_ids = package_manifest
            .provides_interfaces
            .iter()
            .map(|interface| interface.id.as_str())
            .collect::<Vec<_>>();
        let provided_interface_membership = provided_interface_ids.iter().copied().collect();
        let dependency_interface_ids = package_manifest
            .dependencies
            .iter()
            .flat_map(|dependency| dependency.interfaces.iter().map(String::as_str))
            .collect::<Vec<_>>();
        let dependency_interface_membership = dependency_interface_ids.iter().copied().collect();
        let runtime_system_anchors = package_manifest
            .modules
            .iter()
            .filter(|module| module.kind == PluginModuleKind::Runtime)
            .flat_map(|module| {
                module
                    .system_anchors
                    .iter()
                    .map(move |anchor| (module.name.as_str(), anchor.as_str()))
            })
            .collect();

        Self {
            duplicates,
            owned_capabilities,
            runtime_module_names,
            runtime_module_name_membership,
            provided_interface_ids,
            provided_interface_membership,
            dependency_interface_ids,
            dependency_interface_membership,
            runtime_system_anchors,
            identity_rows_indexed,
            membership_probes: Cell::new(0),
        }
    }
}

fn package_identity_row_capacity(package_manifest: &PluginPackageManifest) -> usize {
    let mut rows = package_manifest
        .capabilities
        .len()
        .saturating_add(package_manifest.asset_roots.len())
        .saturating_add(package_manifest.content_roots.len())
        .saturating_add(package_manifest.options.len())
        .saturating_add(package_manifest.event_catalogs.len())
        .saturating_add(package_manifest.components.len())
        .saturating_add(package_manifest.ui_components.len());

    for importer in &package_manifest.asset_importers {
        rows = rows
            .saturating_add(1)
            .saturating_add(importer.required_capabilities.len());
    }
    for dependency in &package_manifest.dependencies {
        rows = rows
            .saturating_add(usize::from(dependency.capability.is_some()))
            .saturating_add(dependency.interfaces.len());
    }
    for status in &package_manifest.capability_statuses {
        rows = rows
            .saturating_add(1)
            .saturating_add(status.bevy_references.len());
    }
    for feature in package_manifest
        .optional_features
        .iter()
        .chain(&package_manifest.feature_extensions)
    {
        rows = rows
            .saturating_add(1)
            .saturating_add(feature.capabilities.len())
            .saturating_add(feature.dependencies.len());
        for module in &feature.modules {
            rows = rows
                .saturating_add(1)
                .saturating_add(module.capabilities.len());
        }
    }
    for interface in &package_manifest.provides_interfaces {
        rows = rows.saturating_add(1);
        for method in &interface.methods {
            rows = rows
                .saturating_add(2)
                .saturating_add(method.required_capabilities.len());
        }
    }
    for module in &package_manifest.modules {
        rows = rows
            .saturating_add(1)
            .saturating_add(module.capabilities.len())
            .saturating_add(module.system_sets.len())
            .saturating_add(module.system_anchors.len());
    }

    rows
}

pub(super) fn index_identity<'a>(
    seen: &mut HashSet<DuplicateIdentity<'a>>,
    duplicates: &mut HashSet<DuplicateOccurrence>,
    identity: DuplicateIdentity<'a>,
    occurrence: DuplicateOccurrence,
    count: &mut usize,
) {
    *count += 1;
    if !seen.insert(identity) {
        duplicates.insert(occurrence);
    }
}
