use crate::{plugin::CapabilityStatus, plugin::PluginMaturity};

use super::super::capability_status::capability_status;
use super::super::BuiltinCatalogDescriptorBuilder;

pub(super) fn is_content_tool_descriptor(package_id: &str) -> bool {
    matches!(package_id, "terrain" | "tilemap_2d" | "prefab_tools")
}

pub(super) fn classify_content_tool_descriptor(
    package_id: &str,
    descriptor: BuiltinCatalogDescriptorBuilder,
) -> BuiltinCatalogDescriptorBuilder {
    descriptor
        .with_maturity(PluginMaturity::Beta)
        .with_capability_status(capability_status(
            format!("runtime.plugin.{package_id}"),
            CapabilityStatus::Partial,
        ))
}
