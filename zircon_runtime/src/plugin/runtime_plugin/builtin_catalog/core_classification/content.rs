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
            runtime_plugin_capability(package_id),
            CapabilityStatus::Partial,
        ))
}

fn runtime_plugin_capability(package_id: &str) -> String {
    const PREFIX: &str = "runtime.plugin.";

    let mut capability = String::with_capacity(PREFIX.len() + package_id.len());
    capability.push_str(PREFIX);
    capability.push_str(package_id);
    capability
}

#[cfg(test)]
#[path = "tests/content.rs"]
mod tests;
