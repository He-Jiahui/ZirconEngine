use crate::{plugin::CapabilityStatus, plugin::PluginMaturity};

use super::advanced_rendering::{
    classify_advanced_render_descriptor, is_advanced_render_descriptor,
};
use super::capability_status::capability_status;
use super::BuiltinCatalogDescriptorBuilder;

pub(super) fn is_render_descriptor(package_id: &str) -> bool {
    package_id == "rendering" || is_advanced_render_descriptor(package_id)
}

pub(super) fn classify_render_descriptor(
    package_id: &str,
    descriptor: BuiltinCatalogDescriptorBuilder,
) -> BuiltinCatalogDescriptorBuilder {
    if is_advanced_render_descriptor(package_id) {
        return classify_advanced_render_descriptor(package_id, descriptor);
    }
    if package_id == "rendering" {
        return descriptor
            .with_maturity(PluginMaturity::Stable)
            .with_capability_status(capability_status(
                "runtime.plugin.rendering",
                CapabilityStatus::Complete,
            ));
    }
    descriptor
}
