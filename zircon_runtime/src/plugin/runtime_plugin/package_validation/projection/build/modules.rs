use std::collections::HashSet;

use crate::plugin::PluginPackageManifest;

use super::super::duplicate_identity::DuplicateIdentity;
use super::super::duplicate_occurrence::DuplicateOccurrence;
use super::index_identity;

pub(super) fn index_package_modules<'a>(
    manifest: &'a PluginPackageManifest,
    seen: &mut HashSet<DuplicateIdentity<'a>>,
    duplicates: &mut HashSet<DuplicateOccurrence>,
    count: &mut usize,
) {
    for (module_index, module) in manifest.modules.iter().enumerate() {
        index_identity(
            seen,
            duplicates,
            DuplicateIdentity::PackageModuleName(&module.name),
            DuplicateOccurrence::PackageModuleName(module_index),
            count,
        );
        for (capability_index, capability) in module.capabilities.iter().enumerate() {
            index_identity(
                seen,
                duplicates,
                DuplicateIdentity::PackageModuleCapability {
                    module: module_index,
                    value: capability,
                },
                DuplicateOccurrence::PackageModuleCapability {
                    module: module_index,
                    capability: capability_index,
                },
                count,
            );
        }
        for (system_set_index, system_set) in module.system_sets.iter().enumerate() {
            index_identity(
                seen,
                duplicates,
                DuplicateIdentity::PackageModuleSystemSet {
                    module: module_index,
                    value: system_set,
                },
                DuplicateOccurrence::PackageModuleSystemSet {
                    module: module_index,
                    system_set: system_set_index,
                },
                count,
            );
        }
        for (system_anchor_index, system_anchor) in module.system_anchors.iter().enumerate() {
            index_identity(
                seen,
                duplicates,
                DuplicateIdentity::PackageModuleSystemAnchor {
                    module: module_index,
                    value: system_anchor,
                },
                DuplicateOccurrence::PackageModuleSystemAnchor {
                    module: module_index,
                    system_anchor: system_anchor_index,
                },
                count,
            );
        }
    }
}
