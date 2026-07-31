use std::collections::HashSet;

use crate::plugin::PluginPackageManifest;

use super::super::duplicate_identity::DuplicateIdentity;
use super::super::duplicate_occurrence::DuplicateOccurrence;
use super::index_identity;

pub(super) fn index_contribution_identities<'a>(
    manifest: &'a PluginPackageManifest,
    seen: &mut HashSet<DuplicateIdentity<'a>>,
    duplicates: &mut HashSet<DuplicateOccurrence>,
    count: &mut usize,
) {
    for (index, option) in manifest.options.iter().enumerate() {
        index_identity(
            seen,
            duplicates,
            DuplicateIdentity::OptionKey(&option.key),
            DuplicateOccurrence::OptionKey(index),
            count,
        );
    }
    for (index, catalog) in manifest.event_catalogs.iter().enumerate() {
        index_identity(
            seen,
            duplicates,
            DuplicateIdentity::EventCatalogNamespace(&catalog.namespace),
            DuplicateOccurrence::EventCatalogNamespace(index),
            count,
        );
    }
    for (index, component) in manifest.components.iter().enumerate() {
        index_identity(
            seen,
            duplicates,
            DuplicateIdentity::ComponentTypeId(&component.type_id),
            DuplicateOccurrence::ComponentTypeId(index),
            count,
        );
    }
    for (index, component) in manifest.ui_components.iter().enumerate() {
        index_identity(
            seen,
            duplicates,
            DuplicateIdentity::UiComponentId(&component.component_id),
            DuplicateOccurrence::UiComponentId(index),
            count,
        );
    }
}
