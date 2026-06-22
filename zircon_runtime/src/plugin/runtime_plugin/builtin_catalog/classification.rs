use crate::plugin::CapabilityStatus;

use super::capability_status::capability_status;
use super::core_classification::{classify_core_descriptor, is_core_descriptor};
use super::importer_classification::{classify_importer_descriptor, is_importer_descriptor};
use super::language::{classify_language_descriptor, is_language_descriptor};
use super::render_classification::{classify_render_descriptor, is_render_descriptor};
use super::BuiltinCatalogDescriptorBuilder;

pub(super) fn classify_descriptor(
    descriptor: BuiltinCatalogDescriptorBuilder,
) -> BuiltinCatalogDescriptorBuilder {
    let package_id = descriptor.package_id().to_string();
    if is_core_descriptor(package_id.as_str()) {
        return classify_core_descriptor(package_id.as_str(), descriptor);
    }
    if is_render_descriptor(package_id.as_str()) {
        return classify_render_descriptor(package_id.as_str(), descriptor);
    }
    if is_importer_descriptor(package_id.as_str()) {
        return classify_importer_descriptor(package_id.as_str(), descriptor);
    }
    if is_language_descriptor(package_id.as_str()) {
        return classify_language_descriptor(package_id.as_str(), descriptor);
    }
    descriptor.with_capability_status(capability_status(
        format!("runtime.plugin.{package_id}"),
        CapabilityStatus::Partial,
    ))
}
