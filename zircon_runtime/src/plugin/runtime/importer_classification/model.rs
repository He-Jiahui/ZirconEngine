use crate::{plugin::CapabilityStatus, plugin::PluginMaturity};

use super::super::capability_status::capability_status;
use super::super::BuiltinCatalogDescriptorBuilder;
use super::capabilities::primary_importer_capability;

pub(super) fn is_model_importer_descriptor(package_id: &str) -> bool {
    matches!(package_id, "gltf_importer" | "obj_importer")
}

pub(super) fn classify_model_importer_descriptor(
    package_id: &str,
    descriptor: BuiltinCatalogDescriptorBuilder,
) -> BuiltinCatalogDescriptorBuilder {
    descriptor
        .with_maturity(PluginMaturity::Stable)
        .with_capability_status(capability_status(
            primary_importer_capability(package_id),
            CapabilityStatus::Partial,
        ))
}
