use crate::plugin::CapabilityStatus;

use super::capability_status::capability_status;
use super::core_classification::{classify_core_descriptor, is_core_descriptor};
use super::importer_classification::{classify_importer_descriptor, is_importer_descriptor};
use super::language::{classify_language_descriptor, is_language_descriptor};
use super::render_classification::{classify_render_descriptor, is_render_descriptor};
use super::IdentifiedBuiltinCatalogDescriptorBuilder;

pub(super) fn classify_descriptor(
    (package_id, descriptor): IdentifiedBuiltinCatalogDescriptorBuilder,
) -> IdentifiedBuiltinCatalogDescriptorBuilder {
    let descriptor = if is_core_descriptor(package_id) {
        classify_core_descriptor(package_id, descriptor)
    } else if is_render_descriptor(package_id) {
        classify_render_descriptor(package_id, descriptor)
    } else if is_importer_descriptor(package_id) {
        classify_importer_descriptor(package_id, descriptor)
    } else if is_language_descriptor(package_id) {
        classify_language_descriptor(package_id, descriptor)
    } else {
        descriptor.with_capability_status(capability_status(
            format!("runtime.plugin.{package_id}"),
            CapabilityStatus::Partial,
        ))
    };
    (package_id, descriptor)
}
