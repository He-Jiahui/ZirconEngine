use crate::{plugin::CapabilityStatus, plugin::PluginMaturity};

use super::super::super::RuntimePluginDescriptor;
use super::super::capability_status::capability_status;
use super::capabilities::primary_importer_capability;

pub(super) fn is_media_importer_descriptor(package_id: &str) -> bool {
    matches!(package_id, "texture_importer" | "audio_importer")
}

pub(super) fn classify_media_importer_descriptor(
    package_id: &str,
    descriptor: RuntimePluginDescriptor,
) -> RuntimePluginDescriptor {
    if package_id == "texture_importer" {
        return descriptor
            .with_maturity(PluginMaturity::Stable)
            .with_capability_status(capability_status(
                "runtime.asset.importer.texture.image",
                CapabilityStatus::Partial,
            ));
    }
    descriptor
        .with_maturity(PluginMaturity::Stable)
        .with_capability_status(capability_status(
            primary_importer_capability(package_id),
            CapabilityStatus::Partial,
        ))
}
